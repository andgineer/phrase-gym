use chrono::{DateTime, SubsecRound, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    // Stored timestamps keep the microsecond precision of existing records.
    fn now(&self) -> DateTime<Utc> {
        Utc::now().trunc_subsecs(6)
    }
}

#[cfg(feature = "test-support")]
#[derive(Debug)]
pub struct ManualClock(std::sync::Mutex<DateTime<Utc>>);

#[cfg(feature = "test-support")]
impl ManualClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self(std::sync::Mutex::new(now))
    }

    pub fn set(&self, now: DateTime<Utc>) {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = now;
    }
}

#[cfg(feature = "test-support")]
impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
