//! Time-related utilities

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

pub struct TimestepLoop {
    timestep_s: f64,
    current_time: f64,
    accum: f64,
    /// Weight given to latest render duration when updating average duration.
    update_weight: f64,
    average_frame_s: f64,
    current_tick: u64,
}

/// Utility structure for tracking FPS and fixed step physics updates.
///
/// Inspired by https://gafferongames.com/post/fix_your_timestep/
///
/// ```
/// use calx::TimestepLoop;
///
/// fn update_physics() {
///     // Physics update here
/// }
///
/// fn render_frame() {
///     // Draw graphics here
/// }
///
/// let mut timestep_loop = TimestepLoop::new(1.0 / 30.0);
///
/// /* loop */ {
///     while timestep_loop.should_update() {
///         update_physics();
///     }
///
///     render_frame();
///     timestep_loop.observe_render();
///     println!("tick: {}, FPS: {}",
///         timestep_loop.current_tick(),
///         1.0 / timestep_loop.average_frame_s());
/// }
/// ```
impl TimestepLoop {
    pub fn new(timestep_s: f64) -> TimestepLoop {
        TimestepLoop {
            timestep_s,
            current_time: time::precise_time_s(),
            accum: 0.0,

            // XXX: Arbitrarily chosen weight parameter
            update_weight: 0.05,

            average_frame_s: 1.0,
            current_tick: 0,
        }
    }

    /// Add to timestep accumulator and update frame duration counter.
    pub fn observe_render(&mut self) {
        let current_time = time::precise_time_s();
        let delta = current_time - self.current_time;
        self.current_time = current_time;
        self.accum += current_time;
        self.average_frame_s =
            self.average_frame_s * (1.0 - self.update_weight) + delta * self.update_weight;
    }

    /// Consume accumulation and return true if accumulation is sufficient for a physics update.
    pub fn should_update(&mut self) -> bool {
        if self.accum < self.timestep_s {
            return false;
        }

        self.accum -= self.timestep_s;
        self.current_tick += 1;
        true
    }

    /// Return remaining time until the next update.
    ///
    /// If this is positive, the caller can sleep between frames
    pub fn time_until_update(&self) -> f64 { self.timestep_s - self.accum }

    /// Return length of the update tick in seconds
    pub fn timestep_s(&self) -> f64 { self.timestep_s }

    /// Return the number of update ticks observed since `TimestepLoop` creation.
    pub fn current_tick(&self) -> u64 { self.current_tick }

    /// Return the average render frame duration in seconds.
    pub fn average_frame_s(&self) -> f64 { self.average_frame_s }
}
