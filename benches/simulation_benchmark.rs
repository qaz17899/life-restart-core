//! Benchmark for simulation performance
//!
//! Target: simulate_full_life should complete in <15ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use life_restart_core::config::{
    AchievementConfig, AgeConfig, EventConfig, EventEffect, TalentConfig, TalentEffect,
};
use life_restart_core::simulator::SimulationEngine;
use std::collections::HashMap;

/// Create a realistic test configuration
fn create_test_config() -> (
    HashMap<i32, TalentConfig>,
    HashMap<i32, EventConfig>,
    HashMap<i32, AgeConfig>,
    HashMap<i32, AchievementConfig>,
    HashMap<String, Vec<life_restart_core::config::JudgeLevel>>,
) {
    let mut talents = HashMap::new();
    
    // Add 100 talents
    for i in 1..=100 {
        talents.insert(
            i,
            TalentConfig {
                id: i,
                name: format!("Talent {}", i),
                description: format!("Description for talent {}", i),
                grade: (i % 4) as i32,
                max_triggers: if i % 10 == 0 { 3 } else { 1 },
                condition: if i % 5 == 0 {
                    Some(format!("AGE>={}", i % 50))
                } else {
                    None
                },
                effect: Some(TalentEffect {
                    chr: if i % 6 == 0 { 1 } else { 0 },
                    int: if i % 6 == 1 { 1 } else { 0 },
                    str_: if i % 6 == 2 { 1 } else { 0 },
                    mny: if i % 6 == 3 { 1 } else { 0 },
                    spr: if i % 6 == 4 { 1 } else { 0 },
                    lif: 0,
                    age: 0,
                    rdm: if i % 6 == 5 { 1 } else { 0 },
                }),
                exclusive: i % 20 == 0,
                exclude: if i % 15 == 0 {
                    Some(vec![(i + 1) % 100 + 1])
                } else {
                    None
                },
                replacement: None,
                status: 0,
            },
        );
    }

    let mut events = HashMap::new();
    
    // Add 500 events
    for i in 1..=500 {
        events.insert(
            i,
            EventConfig {
                id: i,
                event: format!("Event {} happened", i),
                grade: (i % 4) as i32,
                no_random: i % 50 == 0,
                include: if i % 10 == 0 {
                    Some(format!("CHR>{}", i % 10))
                } else {
                    None
                },
                exclude: if i % 20 == 0 {
                    Some(format!("INT<{}", i % 5))
                } else {
                    None
                },
                effect: Some(EventEffect {
                    chr: if i % 7 == 0 { 1 } else { 0 },
                    int: if i % 7 == 1 { 1 } else { 0 },
                    str_: if i % 7 == 2 { 1 } else { 0 },
                    mny: if i % 7 == 3 { 1 } else { 0 },
                    spr: if i % 7 == 4 { 1 } else { 0 },
                    lif: if i % 100 == 0 { -1 } else { 0 },
                    age: 0,
                    rdm: 0,
                }),
                branch: None,
                post_event: None,
            },
        );
    }

    // Add death event
    events.insert(
        999,
        EventConfig {
            id: 999,
            event: "Life ends".to_string(),
            grade: 0,
            no_random: false,
            include: None,
            exclude: None,
            effect: Some(EventEffect {
                chr: 0,
                int: 0,
                str_: 0,
                mny: 0,
                spr: 0,
                lif: -10,
                age: 0,
                rdm: 0,
            }),
            branch: None,
            post_event: None,
        },
    );

    let mut ages = HashMap::new();
    
    // Add age configs for 0-120
    for age in 0..=120 {
        let event_pool: Vec<(i32, f64)> = if age < 100 {
            // Normal events
            (1..=20)
                .map(|i| ((age % 500) + i, 1.0))
                .collect()
        } else {
            // Death events after 100
            vec![(999, 1.0)]
        };

        ages.insert(
            age,
            AgeConfig {
                age,
                talents: if age == 0 {
                    Some(vec![1, 2, 3])
                } else {
                    None
                },
                events: Some(event_pool),
            },
        );
    }

    let mut achievements = HashMap::new();
    
    // Add 50 achievements
    for i in 1..=50 {
        achievements.insert(
            i,
            AchievementConfig {
                id: i,
                name: format!("Achievement {}", i),
                description: format!("Description for achievement {}", i),
                grade: (i % 4) as i32,
                opportunity: match i % 3 {
                    0 => "START".to_string(),
                    1 => "TRAJECTORY".to_string(),
                    _ => "SUMMARY".to_string(),
                },
                condition: format!("HCHR>{}", i % 10),
            },
        );
    }

    let judge_config = HashMap::new();

    (talents, events, ages, achievements, judge_config)
}

fn benchmark_simulation(c: &mut Criterion) {
    let (talents, events, ages, achievements, judge_config) = create_test_config();
    let engine = SimulationEngine::new(talents, events, ages, achievements, judge_config);

    let talent_ids = vec![1, 2, 3];
    let mut properties = HashMap::new();
    properties.insert("CHR".to_string(), 5);
    properties.insert("INT".to_string(), 5);
    properties.insert("STR".to_string(), 5);
    properties.insert("MNY".to_string(), 5);
    let achieved: std::collections::HashSet<i32> = std::collections::HashSet::new();

    c.bench_function("simulate_full_life", |b| {
        b.iter(|| {
            let result = engine.simulate(
                black_box(&talent_ids),
                black_box(&properties),
                black_box(&achieved),
            );
            black_box(result)
        })
    });
}

fn benchmark_game_session(c: &mut Criterion) {
    use life_restart_core::simulator::{default_emoji_map, GameSession};
    use std::sync::Arc;

    let (talents, events, ages, achievements, judge_config) = create_test_config();
    let engine = SimulationEngine::new(talents, events, ages, achievements, judge_config);

    let talent_ids = vec![1, 2, 3];
    let mut properties = HashMap::new();
    properties.insert("CHR".to_string(), 5);
    properties.insert("INT".to_string(), 5);
    properties.insert("STR".to_string(), 5);
    properties.insert("MNY".to_string(), 5);
    let achieved: std::collections::HashSet<i32> = std::collections::HashSet::new();
    let emoji_map = Arc::new(default_emoji_map());

    // Benchmark simulation + GameSession creation (pre-rendering)
    c.bench_function("simulate_with_game_session", |b| {
        b.iter(|| {
            let result = engine.simulate(
                black_box(&talent_ids),
                black_box(&properties),
                black_box(&achieved),
            ).unwrap();
            let session = GameSession::new(result, emoji_map.clone());
            black_box(session)
        })
    });

    // Benchmark just GameSession creation (pre-rendering overhead)
    let result = engine.simulate(&talent_ids, &properties, &achieved).unwrap();
    c.bench_function("game_session_pre_rendering", |b| {
        b.iter(|| {
            let session = GameSession::new(black_box(result.clone()), emoji_map.clone());
            black_box(session)
        })
    });
}

fn benchmark_condition_parsing(c: &mut Criterion) {
    use life_restart_core::condition::cache::clear_cache;
    use life_restart_core::condition::parser::parse;

    let conditions = vec![
        "CHR>5",
        "CHR>5 & INT<10",
        "CHR>5 | INT<10",
        "AGE>=18 & CHR>5 & (TLT?[1001] | EVT?[10001])",
        "HCHR>=10 & HINT>=10 & HSTR>=10",
    ];

    c.bench_function("condition_parsing_cold", |b| {
        b.iter(|| {
            clear_cache();
            for cond in &conditions {
                let _ = black_box(parse(cond));
            }
        })
    });

    c.bench_function("condition_parsing_cached", |b| {
        // Warm up cache
        for cond in &conditions {
            let _ = parse(cond);
        }
        
        b.iter(|| {
            for cond in &conditions {
                let _ = black_box(parse(cond));
            }
        })
    });
}

criterion_group!(benches, benchmark_simulation, benchmark_game_session, benchmark_condition_parsing);
criterion_main!(benches);
