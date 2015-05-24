/**!
 * Easing functions for animated interpolation between values
 */

use std::f32::consts::{FRAC_PI_2};

pub fn linear(t: f32) -> f32 { t }

pub fn quadratic_in(t: f32) -> f32 { t * t }

pub fn quadratic_out(t: f32) -> f32 { -(t * (t - 2.0)) }

pub fn quadratic_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -2.0 * t * t + 4.0 * t - 1.0
    }
}

pub fn cubic_in(t: f32) -> f32 { t * t * t }

pub fn cubic_out(t: f32) -> f32 {
    let u = t - 1.0;
    u * u * u + 1.0
}

pub fn cubic_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let u = 2.0 * t - 2.0;
        0.5 * u * u * u + 1.0
    }
}

pub fn sin_in(t: f32) -> f32 {
    ((t - 1.0) * FRAC_PI_2).sin() + 1.0
}

pub fn sin_out(t: f32) -> f32 {
    (t * FRAC_PI_2).sin()
}

pub fn sin_in_out(t: f32) -> f32 {
    0.5 * (1.0 - (t * FRAC_PI_2).cos())
}

// TODO: Elastic easing
