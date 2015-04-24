use std::ops::{Add, Sub, Mul, Div, Rem};
use num::{NumCast};

// Needed for general "can do math with this" trait that encompasses both
// integers and floats.

pub trait Primitive: Add<Self, Output=Self> + Sub<Self, Output=Self>
    + Mul<Self, Output=Self> + Div<Self, Output=Self> + Rem<Self, Output=Self>
    + PartialEq + Copy + NumCast + PartialOrd + Clone {}

impl Primitive for isize {}
impl Primitive for i8 {}
impl Primitive for i16 {}
impl Primitive for i32 {}
impl Primitive for i64 {}
impl Primitive for f32 {}
impl Primitive for f64 {}

