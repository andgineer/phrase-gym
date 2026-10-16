use std::f64::consts::E;

use chrono::{DateTime, TimeDelta, Utc};

use super::card::{CardState, MemoryState};

pub const DEFAULT_WEIGHTS: [f64; 21] = [
    0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001, 1.8722, 0.1666, 0.796, 1.4835,
    0.0614, 0.2629, 1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542,
];
const STABILITY_MIN: f64 = 0.001;
const MIN_DIFFICULTY: f64 = 1.0;
const MAX_DIFFICULTY: f64 = 10.0;
const MICROS_PER_DAY: i64 = 86_400_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
}

impl Rating {
    fn value(self) -> f64 {
        f64::from(self as u8)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameters {
    pub weights: [f64; 21],
    pub desired_retention: f64,
    pub maximum_interval: i64,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            weights: DEFAULT_WEIGHTS,
            desired_retention: 0.9,
            maximum_interval: 36_500,
        }
    }
}

// FSRS-6 arithmetic without short learning steps or fuzzing; the review policy owns eligibility.
// Powers of e use powf rather than exp to reproduce the reference implementation's rounding.
#[derive(Clone, Debug)]
pub struct Scheduler {
    parameters: Parameters,
    decay: f64,
    factor: f64,
}

impl Scheduler {
    pub fn new(parameters: Parameters) -> Self {
        let decay = -parameters.weights[20];
        let factor = 0.9_f64.powf(1.0 / decay) - 1.0;
        Self {
            parameters,
            decay,
            factor,
        }
    }

    pub fn parameters(&self) -> &Parameters {
        &self.parameters
    }

    pub fn review(
        &self,
        card: Option<&MemoryState>,
        rating: Rating,
        at: DateTime<Utc>,
    ) -> MemoryState {
        let previous = card.cloned().unwrap_or_else(|| MemoryState::new(at));
        let elapsed = previous.last_review.map(|last| whole_days(at - last));
        let (stability, difficulty) = match (previous.stability, previous.difficulty) {
            (Some(stability), Some(difficulty)) => {
                let stability = match elapsed {
                    Some(days) if days < 1 => self.short_term_stability(stability, rating),
                    _ => {
                        let retrievability =
                            elapsed.map_or(0.0, |days| self.retrievability(stability, days.max(0)));
                        self.next_stability(difficulty, stability, retrievability, rating)
                    }
                };
                (stability, self.next_difficulty(difficulty, rating))
            }
            _ => (
                self.initial_stability(rating),
                self.initial_difficulty(rating.value())
                    .clamp(MIN_DIFFICULTY, MAX_DIFFICULTY),
            ),
        };
        MemoryState {
            card_id: previous.card_id,
            state: CardState::Review,
            step: None,
            stability: Some(stability),
            difficulty: Some(difficulty),
            due: at + TimeDelta::days(self.next_interval(stability)),
            last_review: Some(at),
        }
    }

    fn retrievability(&self, stability: f64, elapsed_days: i64) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let elapsed = elapsed_days as f64;
        (1.0 + self.factor * elapsed / stability).powf(self.decay)
    }

    fn initial_stability(&self, rating: Rating) -> f64 {
        self.parameters.weights[rating as usize - 1].max(STABILITY_MIN)
    }

    fn initial_difficulty(&self, rating: f64) -> f64 {
        let w = &self.parameters.weights;
        w[4] - E.powf(w[5] * (rating - 1.0)) + 1.0
    }

    #[allow(clippy::cast_possible_truncation)]
    fn next_interval(&self, stability: f64) -> i64 {
        let retention = self.parameters.desired_retention;
        let days = (stability / self.factor) * (retention.powf(1.0 / self.decay) - 1.0);
        (days.round_ties_even() as i64).clamp(1, self.parameters.maximum_interval)
    }

    fn short_term_stability(&self, stability: f64, rating: Rating) -> f64 {
        let w = &self.parameters.weights;
        let mut increase =
            E.powf(w[17] * ((rating.value() - 3.0) + w[18])) * stability.powf(-w[19]);
        if rating != Rating::Again {
            increase = increase.max(1.0);
        }
        (stability * increase).max(STABILITY_MIN)
    }

    fn next_difficulty(&self, difficulty: f64, rating: Rating) -> f64 {
        let w = &self.parameters.weights;
        let easy = self.initial_difficulty(4.0);
        let delta = -(w[6] * (rating.value() - 3.0));
        let damped = difficulty + (10.0 - difficulty) * delta / 9.0;
        (w[7] * easy + (1.0 - w[7]) * damped).clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
    }

    fn next_stability(
        &self,
        difficulty: f64,
        stability: f64,
        retrievability: f64,
        rating: Rating,
    ) -> f64 {
        let w = &self.parameters.weights;
        let next = if rating == Rating::Again {
            let long_term = w[11]
                * difficulty.powf(-w[12])
                * ((stability + 1.0).powf(w[13]) - 1.0)
                * E.powf((1.0 - retrievability) * w[14]);
            let short_term = stability / E.powf(w[17] * w[18]);
            long_term.min(short_term)
        } else {
            let hard_penalty = if rating == Rating::Hard { w[15] } else { 1.0 };
            stability
                * (1.0
                    + E.powf(w[8])
                        * (11.0 - difficulty)
                        * stability.powf(-w[9])
                        * (E.powf((1.0 - retrievability) * w[10]) - 1.0)
                        * hard_penalty)
        };
        next.max(STABILITY_MIN)
    }
}

// Whole elapsed days rounded toward negative infinity, as the reference timedelta arithmetic does.
fn whole_days(delta: TimeDelta) -> i64 {
    delta
        .num_microseconds()
        .map_or(delta.num_days(), |micros| micros.div_euclid(MICROS_PER_DAY))
}
