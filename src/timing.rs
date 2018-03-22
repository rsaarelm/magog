//! Time-related utilities

use std::thread;
use time;

/// Animation cycle based on system clock.
pub fn cycle_anim(period_s: f64, num_frames: usize) -> usize {
    debug_assert!(period_s > 0.0);
    debug_assert!(num_frames > 0);
    (time::precise_time_s() / period_s) as usize % num_frames
}

/// Time-plot that spikes at given intervals for the given time.
pub fn spike(down_s: f64, up_s: f64) -> bool { time::precise_time_s() % (down_s + up_s) > down_s }

pub fn single_anim(start_s: f64, period_s: f64, num_frames: usize) -> usize {
    debug_assert!(period_s > 0.0);
    debug_assert!(num_frames > 0);
    let mut idx = ((time::precise_time_s() - start_s) / period_s) as i32;
    if idx < 0 {
        idx = 0;
    }
    if idx >= num_frames as i32 {
        idx = num_frames as i32 - 1;
    }

    idx as usize
}
