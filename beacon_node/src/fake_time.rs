//! Traits for testing code that uses [`Instant`] and [`SystemTime`].
//!
//! [`Instant`]:    std::time::Instant
//! [`SystemTime`]: std::time::SystemTime

use core::{ops::Add, time::Duration};
use std::{
    error::Error,
    time::{Instant, SystemTime, SystemTimeError},
};

use thiserror::Error;

pub trait InstantLike: Add<Duration, Output = Self> + Sized {}

pub trait SystemTimeLike: Copy {
    type Error: Error + Send + Sync + 'static;

    const UNIX_EPOCH: Self;

    fn duration_since(&self, earlier: Self) -> Result<Duration, Self::Error>;
}

impl InstantLike for Instant {}

impl SystemTimeLike for SystemTime {
    type Error = SystemTimeError;

    const UNIX_EPOCH: Self = Self::UNIX_EPOCH;

    fn duration_since(&self, earlier: Self) -> Result<Duration, Self::Error> {
        self.duration_since(earlier)
    }
}

/// Time as a [`Duration`] after the Unix epoch.
///
/// Representing time this way lets us avoid reimplementing all the time arithmetic.
/// We cannot represent times before the Unix epoch, but that is not needed in this project.
pub type Timespec = Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FakeInstant(pub Timespec);

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy)]
pub struct FakeSystemTime(pub Timespec);

#[derive(Debug, Error)]
#[error("{0:?}")]
pub struct FakeSystemTimeError(pub Duration);

impl Add<Duration> for FakeInstant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl InstantLike for FakeInstant {}

impl SystemTimeLike for FakeSystemTime {
    type Error = FakeSystemTimeError;

    const UNIX_EPOCH: Self = Self(Duration::from_secs(0));

    fn duration_since(&self, earlier: Self) -> Result<Duration, Self::Error> {
        let later = self.0;
        let earlier = earlier.0;
        later
            .checked_sub(earlier)
            .ok_or_else(|| FakeSystemTimeError(earlier - later))
    }
}

#[cfg(test)]
mod system_time_tests {
    use super::*;

    #[test]
    fn has_excellent_test_coverage() {
        let duration = Duration::from_secs(10000);
        let earlier = <SystemTime as SystemTimeLike>::UNIX_EPOCH;
        let later = <SystemTime as SystemTimeLike>::UNIX_EPOCH + duration;
        let difference = SystemTimeLike::duration_since(&later, earlier).expect("earlier < later");

        assert_eq!(difference, duration);
    }
}
