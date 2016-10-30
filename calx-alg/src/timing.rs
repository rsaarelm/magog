//! Time-related utilities

use time;
use std::thread;
use std::time::Duration;

/// Animation cycle based on system clock.
pub fn cycle_anim<'a, T>(period_s: f64, frames: &'a [T]) -> &'a T {
    assert!(period_s > 0.0);
    assert!(frames.len() > 0);
    let idx = (time::precise_time_s() / period_s) as usize % frames.len();

    &frames[idx]
}

/// Time-plot that spikes at given intervals for the given time.
pub fn spike(down_s: f64, up_s: f64) -> bool { time::precise_time_s() % (down_s + up_s) > down_s }

pub fn single_anim<'a, T>(start_s: f64, period_s: f64, frames: &'a [T]) -> &'a T {
    assert!(period_s > 0.0);
    assert!(frames.len() > 0);
    let mut idx = ((time::precise_time_s() - start_s) / period_s) as i32;
    if idx < 0 {
        idx = 0;
    }
    if idx >= frames.len() as i32 {
        idx = frames.len() as i32 - 1;
    }

    &frames[idx as usize]
}

#[derive(Copy, Clone)]
pub struct Ticker {
    period_s: f64,
    last_t: f64,
}

impl Ticker {
    pub fn new(period_s: f64) -> Ticker {
        Ticker {
            period_s: period_s,
            last_t: time::precise_time_s(),
        }
    }

    fn time_remaining(&mut self) -> Option<f64> {
        let now = time::precise_time_s();
        if now - self.last_t > self.period_s {
            if now - self.last_t > self.period_s * 2.0 {
                // Bring the clock up to speed if running very late.
                self.last_t = now;
            } else {
                self.last_t += self.period_s;
            }
            None
        } else {
            Some(self.period_s - (now - self.last_t))
        }
    }

    pub fn get(&mut self) -> bool { self.time_remaining().is_none() }

    pub fn wait_for_tick(&mut self) {
        match self.time_remaining() {
            Some(t) => {
                thread::sleep(Duration::from_millis((t * 1e3) as u64));
                self.last_t += self.period_s;
            }
            _ => {}
        }
    }
}

#[derive(Copy, Clone)]
pub struct TimePerFrame {
    update_weight: f64,
    start_t: f64,
    pub average: f64,
    pub last: f64,
}

impl TimePerFrame {
    pub fn new(update_weight: f64) -> TimePerFrame {
        assert!(update_weight >= 0.0 && update_weight <= 1.0);
        TimePerFrame {
            update_weight: update_weight,
            start_t: time::precise_time_s(),
            average: 0.0,
            last: 0.0,
        }
    }

    pub fn begin(&mut self) { self.start_t = time::precise_time_s(); }

    pub fn end(&mut self) {
        self.last = time::precise_time_s() - self.start_t;
        self.average = self.update_weight * self.last + (1.0 - self.update_weight) * self.average;
    }
}

/// Exponential moving average duration.
pub struct AverageDuration {
    weight: f64,
    last_time: f64,
    pub value: f64,
}

impl AverageDuration {
    /// Init is the initial value for the duration, somewhere in the scale you
    /// expect the actual values to be. Weight is between 0 and 1 and
    /// indicates how fast the older values should decay. Weight 1.0 causes
    /// old values to decay immediately.
    pub fn new(init: f64, weight: f64) -> AverageDuration {
        assert!(weight > 0.0 && weight <= 1.0);
        AverageDuration {
            weight: weight,
            last_time: time::precise_time_s(),
            value: init,
        }
    }

    pub fn tick(&mut self) {
        let t = time::precise_time_s();
        self.value = self.weight * (t - self.last_time) + (1.0 - self.weight) * self.value;
        self.last_time = t;
    }
}
