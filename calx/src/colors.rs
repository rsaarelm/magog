use serde_derive::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};
use vitral::{Rgba, SRgba};

impl From<Xterm256Color> for Rgba {
    fn from(c: Xterm256Color) -> Rgba { SRgba::from(c).into() }
}

impl From<Rgba> for Xterm256Color {
    fn from(c: Rgba) -> Xterm256Color { SRgba::from(c).into() }
}

/// Base terminal color, no bright colors allowed.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum BaseTermColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl From<BaseTermColor> for u32 {
    fn from(t: BaseTermColor) -> u32 { t as u32 }
}

impl From<BaseTermColor> for SRgba {
    fn from(t: BaseTermColor) -> SRgba { SRgba::from(Xterm256Color(t as u8)) }
}

/// Terminal color, include both dark and bright colors.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct TermColor {
    pub base: BaseTermColor,
    pub is_bright: bool,
}

/// Helper structure that enables using lerp on Term colors.
pub struct TermColorInterpolator(TermColor, f32);

impl Sub for TermColor {
    type Output = TermColorInterpolator;

    fn sub(self, _other: TermColor) -> TermColorInterpolator {
        // This is just a hack to make TermColor work with lerp, so throw away the other color here
        // and make this a thing going towards self-color from anything this is added to later.
        TermColorInterpolator(self, 1.0)
    }
}

impl Mul<f32> for TermColorInterpolator {
    type Output = TermColorInterpolator;

    fn mul(mut self, c: f32) -> TermColorInterpolator {
        self.1 *= c;
        self
    }
}

impl Add<TermColorInterpolator> for TermColor {
    type Output = PseudoTermColor;

    fn add(self, other: TermColorInterpolator) -> PseudoTermColor { self.lerp(&other.0, other.1) }
}

impl From<TermColor> for u32 {
    fn from(t: TermColor) -> u32 { t.base as u32 + if t.is_bright { 8 } else { 0 } }
}

impl From<TermColor> for SRgba {
    fn from(t: TermColor) -> SRgba { SRgba::from(Xterm256Color(u32::from(t) as u8)) }
}

impl TermColor {
    /// Linearly interpolate between two terminal colors.
    ///
    /// Return a gradient pseudocolor value. This can be printed using the unicode gradient
    /// characters and the specified terminal colors.
    pub fn lerp(&self, other: &TermColor, x: f32) -> PseudoTermColor {
        if self.is_bright {
            // Cannot use self as background color.
            if !other.is_bright {
                // But can use the other one, just invert the lerp.
                other.lerp(self, 1.0 - x)
            } else if x < 0.5 {
                // Otherwise just do a hard switch at 50 %, can't gradient.
                PseudoTermColor::Solid(*self)
            } else {
                PseudoTermColor::Solid(*other)
            }
        } else if x < 0.125 {
            PseudoTermColor::Solid(*self)
        } else if x < 0.375 {
            PseudoTermColor::Mixed {
                fore: *other,
                back: self.base,
                mix: ColorMix::Mix25,
            }
        } else if x < 0.5 {
            PseudoTermColor::Mixed {
                fore: *other,
                back: self.base,
                mix: ColorMix::Mix50Low,
            }
        } else if x < 0.625 {
            PseudoTermColor::Mixed {
                fore: *other,
                back: self.base,
                mix: ColorMix::Mix50High,
            }
        } else if x < 0.875 {
            PseudoTermColor::Mixed {
                fore: *other,
                back: self.base,
                mix: ColorMix::Mix75,
            }
        } else {
            PseudoTermColor::Solid(*other)
        }
    }
}

/// Terminal pseudocolor using background and foreground colors.
///
/// Background color must be `BaseColor`. It's assumed that the partial mixes are implemented with
/// the unicode ░▒▓█ gradient characters.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum PseudoTermColor {
    Mixed {
        fore: TermColor,
        back: BaseTermColor,
        mix: ColorMix,
    },
    Solid(TermColor),
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum ColorMix {
    Mix25,
    Mix50Low,
    Mix50High,
    Mix75,
}

impl PseudoTermColor {
    /// Return the halftone character for this pseudocolor.
    pub fn ch(self) -> char {
        match self {
            PseudoTermColor::Mixed {
                mix: ColorMix::Mix25,
                ..
            } => '░',
            PseudoTermColor::Mixed {
                mix: ColorMix::Mix75,
                ..
            } => '▓',
            PseudoTermColor::Mixed { .. } => '▒',
            PseudoTermColor::Solid(_) => '█',
        }
    }

    /// Return the closest color to this pseudocolor.
    pub fn color(self) -> TermColor {
        match self {
            PseudoTermColor::Mixed {
                back: base,
                mix: ColorMix::Mix25,
                ..
            } => TermColor {
                base,
                is_bright: false,
            },
            PseudoTermColor::Mixed {
                back: base,
                mix: ColorMix::Mix50Low,
                ..
            } => TermColor {
                base,
                is_bright: false,
            },
            PseudoTermColor::Mixed {
                fore: c,
                mix: ColorMix::Mix50High,
                ..
            } => c,
            PseudoTermColor::Mixed {
                fore: c,
                mix: ColorMix::Mix75,
                ..
            } => c,
            PseudoTermColor::Solid(c) => c,
        }
    }
}

/// Standard 16 terminal colors.
pub mod term_color {
    use super::BaseTermColor::*;
    use super::TermColor;

    pub const BLACK: TermColor = TermColor {
        base: Black,
        is_bright: false,
    };
    pub const NAVY: TermColor = TermColor {
        base: Blue,
        is_bright: false,
    };
    pub const GREEN: TermColor = TermColor {
        base: Green,
        is_bright: false,
    };
    pub const TEAL: TermColor = TermColor {
        base: Cyan,
        is_bright: false,
    };
    pub const MAROON: TermColor = TermColor {
        base: Red,
        is_bright: false,
    };
    pub const PURPLE: TermColor = TermColor {
        base: Magenta,
        is_bright: false,
    };
    pub const OLIVE: TermColor = TermColor {
        base: Yellow,
        is_bright: false,
    };
    pub const SILVER: TermColor = TermColor {
        base: White,
        is_bright: false,
    };
    pub const GRAY: TermColor = TermColor {
        base: Black,
        is_bright: true,
    };
    pub const BLUE: TermColor = TermColor {
        base: Blue,
        is_bright: true,
    };
    pub const LIME: TermColor = TermColor {
        base: Green,
        is_bright: true,
    };
    pub const AQUA: TermColor = TermColor {
        base: Cyan,
        is_bright: true,
    };
    pub const RED: TermColor = TermColor {
        base: Red,
        is_bright: true,
    };
    pub const FUCHSIA: TermColor = TermColor {
        base: Magenta,
        is_bright: true,
    };
    pub const YELLOW: TermColor = TermColor {
        base: Yellow,
        is_bright: true,
    };
    pub const WHITE: TermColor = TermColor {
        base: White,
        is_bright: true,
    };
}

/// Xterm 256 color palette values
///
/// The values from 16 to 231 are RGB values with 6 steps per channel. The values from 232 to 255
/// are a grayscale gradient. Only color values that have the exact same R, G and B components are guaranteed
/// to use the grayscale area.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Xterm256Color(pub u8);

impl From<Xterm256Color> for SRgba {
    fn from(c: Xterm256Color) -> SRgba {
        // XXX: This should really be a precalculated table...

        // The first 16 are the EGA colors
        if c.0 == 7 {
            SRgba::rgb(192, 192, 192)
        } else if c.0 == 8 {
            SRgba::rgb(128, 128, 128)
        } else if c.0 < 16 {
            let i = if c.0 & 0b1000 != 0 { 255 } else { 128 };
            let r = if c.0 & 0b1 != 0 { i } else { 0 };
            let g = if c.0 & 0b10 != 0 { i } else { 0 };
            let b = if c.0 & 0b100 != 0 { i } else { 0 };

            SRgba::rgb(r * i, g * i, b * i)
        } else if c.0 < 232 {
            fn channel(i: u8) -> u8 { i * 40 + if i > 0 { 55 } else { 0 } }
            // 6^3 RGB space
            let c = c.0 - 16;
            let b = channel(c % 6);
            let g = channel((c / 6) % 6);
            let r = channel(c / 36);

            SRgba::rgb(r, g, b)
        } else {
            // 24 level grayscale slide
            let c = c.0 - 232;
            let c = 8 + 10 * c;
            SRgba::rgb(c, c, c)
        }
    }
}

impl From<SRgba> for Xterm256Color {
    fn from(c: SRgba) -> Xterm256Color {
        // Never convert into the first 16 colors, the user may have configured those to have
        // different RGB values.

        /// Return square of distance to other color
        fn distance2(one: &SRgba, other: &SRgba) -> i32 {
            let x = one.r as i32 - other.r as i32;
            let y = one.g as i32 - other.g as i32;
            let z = one.b as i32 - other.b as i32;
            let w = one.a as i32 - other.a as i32;

            x * x + y * y + z * z + w * w
        }

        fn rgb_channel(c: u8) -> u8 {
            if c < 48 {
                0
            } else if c < 75 {
                1
            } else {
                (c - 35) / 40
            }
        }

        fn gray_channel(c: u8) -> u8 {
            if c < 13 {
                0
            } else if c > 235 {
                23
            } else {
                (c - 3) / 10
            }
        }

        let rgb_color = {
            let r = rgb_channel(c.r);
            let g = rgb_channel(c.g);
            let b = rgb_channel(c.b);

            Xterm256Color(16 + b + g * 6 + r * 36)
        };

        let gray_color = {
            // This is a terrible way to turn any saturated color into grayscale. But unless
            // your color is very gray to begin with, the gray color will have a large error and
            // will lose to the RGB color anyway.
            //
            // Not using SRgba::luma because it involves much more work than this.
            let gray = ((c.r as u32 + c.g as u32 + c.b as u32) / 3) as u8;
            Xterm256Color(232 + gray_channel(gray))
        };

        if distance2(&c, &rgb_color.into()) < distance2(&c, &gray_color.into()) {
            rgb_color
        } else {
            gray_color
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_xterm256() {
        use super::{SRgba, Xterm256Color};
        for i in 16..256 {
            let c = Xterm256Color(i as u8);
            let rgb: SRgba = c.into();
            assert_eq!(c, Xterm256Color::from(rgb));
        }
    }
}
