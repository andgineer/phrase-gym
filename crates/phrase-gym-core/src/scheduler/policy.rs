use chrono::{DateTime, Utc};
use phrase_gym_contracts::practice::RecallRating;

use super::card::{MemoryState, StateError};
use super::fsrs::{Parameters, Rating, Scheduler};

// Sessions and schedules pin this identity; keep it while the arithmetic stays fixture-equivalent.
pub const POLICY_VERSION: &str = "fsrs-6.3.2-retention90-v1";

#[derive(Clone, Debug)]
pub struct ReviewPolicy {
    scheduler: Scheduler,
}

impl Default for ReviewPolicy {
    fn default() -> Self {
        Self {
            scheduler: Scheduler::new(Parameters::default()),
        }
    }
}

impl ReviewPolicy {
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn review(
        &self,
        state: Option<&str>,
        rating: RecallRating,
        now: DateTime<Utc>,
    ) -> Result<(String, DateTime<Utc>), StateError> {
        let previous = state.map(MemoryState::from_json).transpose()?;
        let updated = self
            .scheduler
            .review(previous.as_ref(), fsrs_rating(rating), now);
        Ok((updated.to_json(), updated.due))
    }
}

fn fsrs_rating(rating: RecallRating) -> Rating {
    match rating {
        RecallRating::NotYet => Rating::Again,
        RecallRating::WithEffort => Rating::Hard,
        RecallRating::Recalled => Rating::Good,
    }
}
