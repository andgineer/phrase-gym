use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::timestamps;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardState {
    Learning = 1,
    Review = 2,
    Relearning = 3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryState {
    pub card_id: i64,
    pub state: CardState,
    pub step: Option<u32>,
    pub stability: Option<f64>,
    pub difficulty: Option<f64>,
    pub due: DateTime<Utc>,
    pub last_review: Option<DateTime<Utc>>,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("memory state is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("memory state has an invalid timestamp: {0}")]
    Timestamp(#[from] chrono::ParseError),
    #[error("memory state has unknown card state {0}")]
    UnknownState(u8),
}

// Field names match memory states already stored in target schedules.
#[derive(Serialize, Deserialize)]
struct StoredState {
    card_id: i64,
    difficulty: Option<f64>,
    due: String,
    last_review: Option<String>,
    stability: Option<f64>,
    state: u8,
    step: Option<u32>,
}

impl MemoryState {
    pub fn new(due: DateTime<Utc>) -> Self {
        Self {
            card_id: 0,
            state: CardState::Learning,
            step: Some(0),
            stability: None,
            difficulty: None,
            due,
            last_review: None,
        }
    }

    pub fn from_json(text: &str) -> Result<Self, StateError> {
        let stored: StoredState = serde_json::from_str(text)?;
        let state = match stored.state {
            1 => CardState::Learning,
            2 => CardState::Review,
            3 => CardState::Relearning,
            other => return Err(StateError::UnknownState(other)),
        };
        let last_review = stored
            .last_review
            .as_deref()
            .map(timestamps::parse)
            .transpose()?;
        // A zero value was read back as absent by the implementation that wrote these states.
        let present = |value: Option<f64>| value.filter(|number| *number != 0.0);
        Ok(Self {
            card_id: stored.card_id,
            state,
            step: stored.step.or((state == CardState::Learning).then_some(0)),
            stability: present(stored.stability),
            difficulty: present(stored.difficulty),
            due: timestamps::parse(&stored.due)?,
            last_review,
        })
    }

    pub fn to_json(&self) -> String {
        let stored = StoredState {
            card_id: self.card_id,
            difficulty: self.difficulty,
            due: timestamps::isoformat(self.due),
            last_review: self.last_review.map(timestamps::isoformat),
            stability: self.stability,
            state: self.state as u8,
            step: self.step,
        };
        serde_json::to_string(&stored).unwrap_or_else(|_| unreachable!("plain fields serialize"))
    }
}
