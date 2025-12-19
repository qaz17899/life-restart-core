//! Event processing logic

use crate::condition::cache::check_condition;
use crate::config::{EventConfig, EventEffect};
use std::collections::HashMap;

/// Result of processing an event
#[derive(Debug, Clone)]
pub struct EventResult {
    pub event_id: i32,
    pub description: String,
    pub grade: i32,
    pub effect: Option<EventEffect>,
    pub next_event_id: Option<i32>,
    pub post_event: Option<String>,
}

/// Process an event and determine the result
pub fn process_event(
    event_id: i32,
    events: &HashMap<i32, EventConfig>,
    state: &crate::property::PropertyState,
) -> Option<EventResult> {
    let event = events.get(&event_id)?;

    // Check branch conditions
    if let Some(ref branches) = event.branch {
        for branch in branches {
            if check_condition(&branch.condition, state).unwrap_or(false) {
                return Some(EventResult {
                    event_id,
                    description: event.event.clone(),
                    grade: event.grade,
                    effect: event.effect.clone(),
                    next_event_id: Some(branch.event_id),
                    post_event: None,
                });
            }
        }
    }

    // No branch matched or no branches
    Some(EventResult {
        event_id,
        description: event.event.clone(),
        grade: event.grade,
        effect: event.effect.clone(),
        next_event_id: None,
        post_event: event.post_event.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EventBranch;
    use crate::property::PropertyState;

    #[test]
    fn test_process_simple_event() {
        let mut events = HashMap::new();
        events.insert(
            1,
            EventConfig {
                id: 1,
                event: "Test event".to_string(),
                grade: 1,
                no_random: false,
                include: None,
                exclude: None,
                effect: None,
                branch: None,
                post_event: Some("Post text".to_string()),
            },
        );

        let state = PropertyState::default();
        let result = process_event(1, &events, &state).unwrap();

        assert_eq!(result.event_id, 1);
        assert_eq!(result.description, "Test event");
        assert_eq!(result.grade, 1);
        assert!(result.next_event_id.is_none());
        assert_eq!(result.post_event, Some("Post text".to_string()));
    }

    #[test]
    fn test_process_event_with_branch() {
        let mut events = HashMap::new();
        events.insert(
            1,
            EventConfig {
                id: 1,
                event: "Test event".to_string(),
                grade: 1,
                no_random: false,
                include: None,
                exclude: None,
                effect: None,
                branch: Some(vec![EventBranch {
                    condition: "CHR>5".to_string(),
                    event_id: 2,
                }]),
                post_event: None,
            },
        );

        let state = PropertyState {
            chr: 10,
            ..Default::default()
        };
        let result = process_event(1, &events, &state).unwrap();

        assert_eq!(result.next_event_id, Some(2));
    }
}
