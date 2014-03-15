use time;

pub fn cycle_anim<'a, T>(period_s: f64, frames: &'a [T]) -> &'a T {
    assert!(period_s > 0.0);
    assert!(frames.len() > 0);
    let idx = (time::precise_time_s() / period_s) as uint % frames.len();

    &frames[idx]
}

pub fn single_anim<'a, T>(start_s: f64, period_s: f64, frames: &'a [T]) -> &'a T {
    assert!(period_s > 0.0);
    assert!(frames.len() > 0);
    let mut idx = ((time::precise_time_s() - start_s) / period_s) as int;
    if idx < 0 { idx = 0; }
    if idx >= frames.len() as int { idx = frames.len() as int - 1; }

    &frames[idx]
}
