use std::ops::{Add, Sub, Mul, Div, Neg, Rem};
use std::num::{NumCast};

// Needed for general "can do math with this" trait that encompasses both
// integers and floats.

pub trait Primitive: Add<Self, Output=Self> + Sub<Self, Output=Self>
    + Mul<Self, Output=Self> + Div<Self, Output=Self> + Rem<Self, Output=Self>
    + Neg + PartialEq + Copy + NumCast + PartialOrd + Clone {}

impl Primitive for usize {}
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for isize {}
impl Primitive for i8 {}
impl Primitive for i16 {}
impl Primitive for i32 {}
impl Primitive for i64 {}
impl Primitive for f32 {}
impl Primitive for f64 {}

