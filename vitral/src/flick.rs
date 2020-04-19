use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

/// Conversion factor for the flick time unit
pub const FLICKS_PER_SECOND: i64 = 705_600_000;

/// Flick time unit.
///
/// See https://github.com/OculusVR/Flicks
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Flick(pub i64);

fn precise_time_ns() -> i64 {
    let duration = time::OffsetDateTime::now() - time::OffsetDateTime::unix_epoch();
    duration.whole_nanoseconds() as i64
}

impl Flick {
    pub fn from_seconds(seconds: f64) -> Flick {
        Flick((seconds * FLICKS_PER_SECOND as f64) as i64)
    }

    pub fn from_nanoseconds(nanos: i64) -> Flick { Flick((nanos as i128 * 7056 / 10_000) as i64) }

    /// Return current time in Flicks since an unspecified epoch.
    pub fn now() -> Flick { Flick::from_nanoseconds(precise_time_ns()) }
}

impl fmt::Display for Flick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3} s", self.0 as f32 / FLICKS_PER_SECOND as f32)
    }
}

impl From<Duration> for Flick {
    fn from(d: Duration) -> Flick {
        let nano = d.as_secs() as i64 * 1_000_000_000 + d.subsec_nanos() as i64;
        Flick::from_nanoseconds(nano)
    }
}

impl From<Flick> for Duration {
    fn from(f: Flick) -> Duration {
        let secs = (f.0 / FLICKS_PER_SECOND) as u64;
        let nanos = ((f.0 % FLICKS_PER_SECOND) * 1_000_000_000 / FLICKS_PER_SECOND) as u32;
        Duration::new(secs, nanos)
    }
}

impl<T: Into<Flick>> Add<T> for Flick {
    type Output = Flick;

    fn add(self, rhs: T) -> Flick {
        Flick(
            self.0
                .checked_add(rhs.into().0)
                .expect("overflow when adding flicks"),
        )
    }
}

impl<T: Into<Flick>> AddAssign<T> for Flick {
    fn add_assign(&mut self, rhs: T) { *self = *self + rhs.into(); }
}

impl<T: Into<Flick>> Sub<T> for Flick {
    type Output = Flick;

    fn sub(self, rhs: T) -> Flick {
        Flick(
            self.0
                .checked_sub(rhs.into().0)
                .expect("overflow when subtracting flicks"),
        )
    }
}

impl<T: Into<Flick>> SubAssign<T> for Flick {
    fn sub_assign(&mut self, rhs: T) { *self = *self - rhs.into(); }
}
