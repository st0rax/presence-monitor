//! Pure presence state machine.
//!
//! Mirrors the PowerShell logic: the persisted state starts as `present =
//! null`. The first observation establishes the initial state without
//! emitting a transition. Afterwards, any change of the boolean presence
//! value emits a transition. No debounce/hysteresis was present in the
//! original, so none is added here (a single ping decides each cycle).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Persisted presence state (schema-compatible with the original `state.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresenceState {
    /// `None` = no observation yet; `Some(true/false)` = present/absent.
    pub present: Option<bool>,
    /// ISO-8601 timestamp of the last state change (or initial set).
    #[serde(rename = "lastTransition")]
    pub last_transition: Option<String>,
}

/// Outcome of feeding one observation into the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// First observation — initial state set, no transition action.
    Initial { present: bool },
    /// Presence unchanged since last observation.
    Unchanged { present: bool },
    /// Presence changed.
    Changed { from: bool, to: bool },
}

impl Transition {
    /// True only for `absent -> present`, which triggers the greeting flow.
    pub fn is_arrival(&self) -> bool {
        matches!(
            self,
            Transition::Changed {
                from: false,
                to: true
            }
        )
    }
}

impl PresenceState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from `state.json` if present, otherwise a fresh (null) state.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::new(),
        }
    }

    /// Persist to `state.json`.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).expect("state serializes");
        std::fs::write(path, json)
    }

    /// Feed a new presence observation and compute the transition. The state
    /// is mutated (and `last_transition` stamped) for `Initial`/`Changed`;
    /// `Unchanged` leaves the state as-is.
    pub fn observe(&mut self, present: bool, now: DateTime<Utc>) -> Transition {
        match self.present {
            None => {
                self.present = Some(present);
                self.last_transition = Some(now.to_rfc3339());
                Transition::Initial { present }
            }
            Some(prev) if prev == present => Transition::Unchanged { present },
            Some(prev) => {
                self.present = Some(present);
                self.last_transition = Some(now.to_rfc3339());
                Transition::Changed {
                    from: prev,
                    to: present,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn first_observation_is_initial_no_action() {
        let mut s = PresenceState::new();
        let tr = s.observe(true, t0());
        assert_eq!(tr, Transition::Initial { present: true });
        assert!(!tr.is_arrival());
        assert_eq!(s.present, Some(true));
        assert!(s.last_transition.is_some());
    }

    #[test]
    fn repeated_same_value_is_unchanged() {
        let mut s = PresenceState::new();
        s.observe(false, t0());
        let stamp = s.last_transition.clone();
        let tr = s.observe(false, t0());
        assert_eq!(tr, Transition::Unchanged { present: false });
        // Unchanged must not restamp the transition time.
        assert_eq!(s.last_transition, stamp);
    }

    #[test]
    fn absent_to_present_is_arrival() {
        let mut s = PresenceState::new();
        s.observe(false, t0());
        let tr = s.observe(true, t0());
        assert_eq!(
            tr,
            Transition::Changed {
                from: false,
                to: true
            }
        );
        assert!(tr.is_arrival());
    }

    #[test]
    fn present_to_absent_is_change_but_not_arrival() {
        let mut s = PresenceState::new();
        s.observe(true, t0());
        let tr = s.observe(false, t0());
        assert_eq!(
            tr,
            Transition::Changed {
                from: true,
                to: false
            }
        );
        assert!(!tr.is_arrival());
    }

    #[test]
    fn full_sequence_of_transitions() {
        let mut s = PresenceState::new();
        // present:null -> observe true (initial), true (unchanged),
        // false (change), false (unchanged), true (arrival)
        assert!(matches!(s.observe(true, t0()), Transition::Initial { .. }));
        assert!(matches!(
            s.observe(true, t0()),
            Transition::Unchanged { .. }
        ));
        assert!(matches!(s.observe(false, t0()), Transition::Changed { .. }));
        assert!(matches!(
            s.observe(false, t0()),
            Transition::Unchanged { .. }
        ));
        assert!(s.observe(true, t0()).is_arrival());
    }

    #[test]
    fn state_roundtrips_through_json() {
        let mut s = PresenceState::new();
        s.observe(true, t0());
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("lastTransition"));
        let back: PresenceState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.present, Some(true));
    }
}
