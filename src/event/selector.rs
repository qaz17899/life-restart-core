//! Event selection logic - Optimized version

use crate::condition::cache::check_condition;
use crate::config::EventConfig;
use crate::property::PropertyState;
use rand::Rng;
use std::collections::HashMap;

/// Select an event from the event pool based on conditions and weights
#[inline]
pub fn select_event(
    event_pool: &[(i32, f64)],
    events: &HashMap<i32, EventConfig>,
    state: &PropertyState,
) -> Option<i32> {
    // Pre-allocate with expected capacity
    let mut available: Vec<(i32, f64)> = Vec::with_capacity(event_pool.len());
    let mut total_weight: f64 = 0.0;

    // Single pass: filter and accumulate weight
    for (event_id, weight) in event_pool {
        if let Some(event) = events.get(event_id) {
            // NoRandom events don't participate in random selection
            if event.no_random {
                continue;
            }

            // Check exclude condition
            if let Some(ref exclude) = event.exclude {
                if check_condition(exclude, state).unwrap_or(false) {
                    continue;
                }
            }

            // Check include condition
            if let Some(ref include) = event.include {
                if !check_condition(include, state).unwrap_or(true) {
                    continue;
                }
            }

            available.push((*event_id, *weight));
            total_weight += weight;
        }
    }

    if available.is_empty() || total_weight <= 0.0 {
        return None;
    }

    // Weighted random selection - inline for performance
    let mut rng = rand::thread_rng();
    let mut random_value = rng.gen::<f64>() * total_weight;

    for (id, weight) in &available {
        random_value -= weight;
        if random_value <= 0.0 {
            return Some(*id);
        }
    }

    // Fallback to last item
    available.last().map(|(id, _)| *id)
}

/// Perform weighted random selection - optimized version
#[inline]
pub fn weighted_random(items: &[(i32, f64)]) -> Option<i32> {
    if items.is_empty() {
        return None;
    }

    let total_weight: f64 = items.iter().map(|(_, w)| w).sum();
    if total_weight <= 0.0 {
        return None;
    }

    let mut rng = rand::thread_rng();
    let mut random_value = rng.gen::<f64>() * total_weight;

    for (id, weight) in items {
        random_value -= weight;
        if random_value <= 0.0 {
            return Some(*id);
        }
    }

    // Fallback to last item
    items.last().map(|(id, _)| *id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_random_single() {
        let items = vec![(1, 1.0)];
        let result = weighted_random(&items);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_weighted_random_empty() {
        let items: Vec<(i32, f64)> = vec![];
        let result = weighted_random(&items);
        assert_eq!(result, None);
    }

    #[test]
    fn test_weighted_random_distribution() {
        let items = vec![(1, 1.0), (2, 1.0)];
        let mut counts = [0, 0];

        for _ in 0..1000 {
            if let Some(id) = weighted_random(&items) {
                counts[(id - 1) as usize] += 1;
            }
        }

        // Both should be selected roughly equally (within 20% tolerance)
        let ratio = counts[0] as f64 / counts[1] as f64;
        assert!(ratio > 0.6 && ratio < 1.4);
    }
}
