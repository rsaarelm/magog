use image;
use num::Num;
use std::fmt;
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

/// Color in sRGB color space.
///
/// This is the physical color definition on computer monitors, also the
/// color format most often used when writing out RGB values of computer
/// graphics colors.
///
/// Valid string representations for a sRGB value are case-insesitive
/// SVG color names ("Green", "powderblue") and hex values in the form
/// of `#RGB`, `#RGBA`, `#RRGGBB` and `#RRGGBBAA` (with or without an
/// alpha channel and with 4 or 8 bits per channel). "RED", "red",
/// "#F00", "#F00F", "#FF0000" and "#FF0000FF" all correspond to the
/// same opaque pure red color.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct SRgba {
    /// sRGB red component
    pub r: u8,
    /// sRGB green component
    pub g: u8,
    /// sRGB blue component
    pub b: u8,
    /// sRGB alpha channel
    pub a: u8,
}

impl SRgba {
    /// Create new color with opaque alpha channel.
    pub fn rgb(r: u8, g: u8, b: u8) -> SRgba { SRgba::new(r, g, b, 255) }

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> SRgba {
        SRgba {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }

    /// Return square of distance to other color
    fn distance2(&self, other: &SRgba) -> i32 {
        let x = self.r as i32 - other.r as i32;
        let y = self.g as i32 - other.g as i32;
        let z = self.b as i32 - other.b as i32;
        let w = self.a as i32 - other.a as i32;

        x * x + y * y + z * z + w * w
    }

    pub fn luma(&self) -> u8 {
        // This is faster, but doesn't quite match with linear RGB luma
        //(self.r as f32 * 0.2126 + self.g as f32 * 0.7152 + self.b as f32 * 0.0722) as u8

        (to_srgb(Rgba::from(*self).luma()) * 255.0).round() as u8
    }
}

impl fmt::Display for SRgba {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "#{:02X}{:02X}{:02X}{:02X}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl From<[u8; 4]> for SRgba {
    fn from(c: [u8; 4]) -> SRgba {
        SRgba {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        }
    }
}

impl From<[u8; 3]> for SRgba {
    fn from(c: [u8; 3]) -> SRgba {
        SRgba {
            r: c[0],
            g: c[1],
            b: c[2],
            a: 255,
        }
    }
}

impl From<u32> for SRgba {
    fn from(c: u32) -> SRgba {
        SRgba::new((c >> 24) as u8, (c >> 16) as u8, (c >> 8) as u8, c as u8)
    }
}

impl From<Rgba> for SRgba {
    fn from(c: Rgba) -> SRgba {
        SRgba::new(
            (to_srgb(c.r) * 255.0).round() as u8,
            (to_srgb(c.g) * 255.0).round() as u8,
            (to_srgb(c.b) * 255.0).round() as u8,
            (to_srgb(c.a) * 255.0).round() as u8,
        )
    }
}

impl From<image::Rgba<u8>> for SRgba {
    fn from(c: image::Rgba<u8>) -> SRgba { SRgba::new(c.data[0], c.data[1], c.data[2], c.data[3]) }
}

impl From<SRgba> for image::Rgba<u8> {
    fn from(c: SRgba) -> image::Rgba<u8> {
        image::Rgba {
            data: [c.r, c.g, c.b, c.a],
        }
    }
}

impl FromStr for SRgba {
    type Err = ();

    fn from_str(s: &str) -> Result<SRgba, ()> {
        if let Some(idx) = parse_color_name(&s.to_string().to_ascii_uppercase()[..]) {
            return Ok(NAMED_COLORS[idx].2);
        }

        if s.starts_with('#') {
            let s = &s[1..];

            // Hex digits per color channel, either 1 or 2. Single digit values
            // get doubled for the color, #420 becomes #442200.
            let digits: usize;

            // Does the color include the alpha channel. If not, assume alpha is
            // fully opaque.
            let alpha: bool;

            match s.len() {
                3 => {
                    digits = 1;
                    alpha = false;
                }
                4 => {
                    digits = 1;
                    alpha = true;
                }
                6 => {
                    digits = 2;
                    alpha = false;
                }
                8 => {
                    digits = 2;
                    alpha = true;
                }
                _ => {
                    return Err(());
                }
            }

            debug_assert!(digits == 1 || digits == 2);

            let r = Num::from_str_radix(&s[0..(digits)], 16);
            let g = Num::from_str_radix(&s[(digits)..(2 * digits)], 16);
            let b = Num::from_str_radix(&s[(2 * digits)..(3 * digits)], 16);
            let a = if alpha {
                Num::from_str_radix(&s[(3 * digits)..(4 * digits)], 16)
            } else if digits == 1 {
                Ok(0xFu8)
            } else {
                Ok(0xFFu8)
            };

            return match (r, g, b, a) {
                (Ok(mut r), Ok(mut g), Ok(mut b), Ok(mut a)) => {
                    if digits == 1 {
                        r += r << 4;
                        g += g << 4;
                        b += b << 4;
                        a += a << 4;
                    }

                    Ok(SRgba::new(r, g, b, a))
                }
                _ => Err(()),
            };
        }

        Err(())
    }
}

/// Color in linear color space.
///
/// This is the canonical color representation that the rendering engine
/// expects to get.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Rgba {
    /// Linear red component
    pub r: f32,
    /// Linear green component
    pub g: f32,
    /// Linear blue component
    pub b: f32,
    /// Alpha channel
    pub a: f32,
}

impl Rgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }

    pub fn luma(&self) -> f32 { self.r * 0.2126 + self.g * 0.7152 + self.b * 0.0722 }

    /// Turn color to monochrome preserving brightness.
    pub fn to_monochrome(&self) -> Rgba {
        let luma = self.r * 0.2126 + self.g * 0.7152 + self.b * 0.0722;
        Rgba::new(luma, luma, luma, self.a)
    }
}

impl FromStr for Rgba {
    type Err = ();

    fn from_str(s: &str) -> Result<Rgba, ()> {
        match SRgba::from_str(s) {
            Ok(a) => Ok(a.into()),
            Err(e) => Err(e),
        }
    }
}

impl From<SRgba> for Rgba {
    fn from(c: SRgba) -> Rgba {
        Rgba::new(
            to_linear(c.r as f32 / 255.0),
            to_linear(c.g as f32 / 255.0),
            to_linear(c.b as f32 / 255.0),
            to_linear(c.a as f32 / 255.0),
        )
    }
}

impl From<[f32; 4]> for Rgba {
    fn from(c: [f32; 4]) -> Rgba { Rgba::new(c[0], c[1], c[2], c[3]) }
}

impl From<[f32; 3]> for Rgba {
    fn from(c: [f32; 3]) -> Rgba { Rgba::new(c[0], c[1], c[2], 1.0) }
}

impl From<u32> for Rgba {
    fn from(c: u32) -> Rgba { SRgba::from(c).into() }
}

impl From<Rgba> for [f32; 4] {
    fn from(c: Rgba) -> [f32; 4] { [c.r, c.g, c.b, c.a] }
}

impl From<Xterm256Color> for Rgba {
    fn from(c: Xterm256Color) -> Rgba { SRgba::from(c).into() }
}

impl From<Rgba> for Xterm256Color {
    fn from(c: Rgba) -> Xterm256Color { SRgba::from(c).into() }
}

impl Add<Rgba> for Rgba {
    type Output = Rgba;
    fn add(self, rhs: Rgba) -> Rgba {
        Rgba::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Sub<Rgba> for Rgba {
    type Output = Rgba;
    fn sub(self, rhs: Rgba) -> Rgba {
        Rgba::new(
            self.r - rhs.r,
            self.g - rhs.g,
            self.b - rhs.b,
            self.a - rhs.a,
        )
    }
}

impl Mul<f32> for Rgba {
    type Output = Rgba;
    fn mul(self, rhs: f32) -> Rgba { Rgba::new(self.r * rhs, self.g * rhs, self.b * rhs, self.a) }
}

impl Mul<Rgba> for Rgba {
    type Output = Rgba;
    fn mul(self, rhs: Rgba) -> Rgba {
        Rgba::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}

#[inline]
pub fn to_linear(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / (1.055)).powf(2.4)
    }
}

#[inline]
pub fn to_srgb(linear: f32) -> f32 {
    if linear < 0.0031308 {
        12.92 * linear
    } else {
        (1.0 + 0.055) * linear.powf(1.0 / 2.4) - 0.055
    }
}

macro_rules! color_constants {
    {
        $($name:ident = ([$sr:expr, $sg:expr, $sb:expr], [$r:expr, $g:expr, $b:expr]),)+
    } => {
        // XXX: Trying to use the count_exprs! hack to get the array
        // size for NAMED_COLORS hits macro recursion limit. Just
        // hardcoding the size since the dataset is pretty stable.

        /// Named color constant table.
        pub static NAMED_COLORS: [(&'static str, Rgba, SRgba); 155] = [
            $((stringify!($name), Rgba { r: $r, g: $g, b: $b, a: 1.0 }, SRgba { r: $sr, g: $sg, b: $sb, a: 0xFF }),)+
        ];

        mod _color_name_to_integer {
            #[allow(non_camel_case_types)]

            pub enum ColorEnum {
                $($name,)+
            }
        }

        /// Return an index to `NAMED_COLORS`.
        fn parse_color_name(upper_case_name: &str) -> Option<usize> {
            match upper_case_name {
                $(stringify!($name) => Some(_color_name_to_integer::ColorEnum::$name as usize),)+
                _ => None
            }
        }

        pub mod scolor {
            /*! Srgba color constants */
            use super::SRgba;
            $(pub static $name: SRgba = SRgba { r: $sr, g: $sg, b: $sb, a: 0xFF };)+
        }

        pub mod color {
            /*! Linear rgba color constants */
            use super::Rgba;
            $(pub static $name: Rgba = Rgba { r: $r, g: $g, b: $b, a: 1.0 };)+
        }
    }
}

color_constants! {
    ALICEBLUE            = ([0xF0, 0xF8, 0xFF], [0.8714, 0.9387, 1.0]),
    ANTIQUEWHITE         = ([0xFA, 0xEB, 0xD7], [0.956, 0.8308, 0.6795]),
    AQUA                 = ([0x00, 0xFF, 0xFF], [0.0, 1.0, 1.0]),
    AQUAMARINE           = ([0x7F, 0xFF, 0xD4], [0.2122, 1.0, 0.6584]),
    AZURE                = ([0xF0, 0xFF, 0xFF], [0.8714, 1.0, 1.0]),
    BEIGE                = ([0xF5, 0xF5, 0xDC], [0.9131, 0.9131, 0.7157]),
    BISQUE               = ([0xFF, 0xE4, 0xC4], [1.0, 0.7758, 0.552]),
    BLACK                = ([0x00, 0x00, 0x00], [0.0, 0.0, 0.0]),
    BLANCHEDALMOND       = ([0xFF, 0xEB, 0xCD], [1.0, 0.8308, 0.6105]),
    BLUE                 = ([0x00, 0x00, 0xFF], [0.0, 0.0, 1.0]),
    BLUEVIOLET           = ([0x8A, 0x2B, 0xE2], [0.2542, 0.02416, 0.7605]),
    BROWN                = ([0xA5, 0x2A, 0x2A], [0.3763, 0.02315, 0.02315]),
    BURLYWOOD            = ([0xDE, 0xB8, 0x87], [0.7305, 0.4793, 0.2423]),
    CADETBLUE            = ([0x5F, 0x9E, 0xA0], [0.1144, 0.3419, 0.3515]),
    CHARTREUSE           = ([0x7F, 0xFF, 0x00], [0.2122, 1.0, 0.0]),
    CHOCOLATE            = ([0xD2, 0x69, 0x1E], [0.6445, 0.1413, 0.01298]),
    CORAL                = ([0xFF, 0x7F, 0x50], [1.0, 0.2122, 0.08022]),
    CORNFLOWERBLUE       = ([0x64, 0x95, 0xED], [0.1274, 0.3005, 0.8469]),
    CORNSILK             = ([0xFF, 0xF8, 0xDC], [1.0, 0.9387, 0.7157]),
    CRIMSON              = ([0xDC, 0x14, 0x3C], [0.7157, 0.006995, 0.04519]),
    CYAN                 = ([0x00, 0xFF, 0xFF], [0.0, 1.0, 1.0]),
    DARKBLUE             = ([0x00, 0x00, 0x8B], [0.0, 0.0, 0.2582]),
    DARKCYAN             = ([0x00, 0x8B, 0x8B], [0.0, 0.2582, 0.2582]),
    DARKGOLDENROD        = ([0xB8, 0x86, 0x0B], [0.4793, 0.2384, 0.003347]),
    DARKGRAY             = ([0xA9, 0xA9, 0xA9], [0.3968, 0.3968, 0.3968]),
    DARKGREEN            = ([0x00, 0x64, 0x00], [0.0, 0.1274, 0.0]),
    DARKKHAKI            = ([0xBD, 0xB7, 0x6B], [0.5089, 0.4735, 0.147]),
    DARKMAGENTA          = ([0x8B, 0x00, 0x8B], [0.2582, 0.0, 0.2582]),
    DARKOLIVEGREEN       = ([0x55, 0x6B, 0x2F], [0.09084, 0.147, 0.02843]),
    DARKORANGE           = ([0xFF, 0x8C, 0x00], [1.0, 0.2623, 0.0]),
    DARKORCHID           = ([0x99, 0x32, 0xCC], [0.3185, 0.0319, 0.6038]),
    DARKRED              = ([0x8B, 0x00, 0x00], [0.2582, 0.0, 0.0]),
    DARKSALMON           = ([0xE9, 0x96, 0x7A], [0.8148, 0.305, 0.1946]),
    DARKSEAGREEN         = ([0x8F, 0xBC, 0x8F], [0.2747, 0.5029, 0.2747]),
    DARKSLATEBLUE        = ([0x48, 0x3D, 0x8B], [0.0648, 0.04667, 0.2582]),
    DARKSLATEGRAY        = ([0x2F, 0x4F, 0x4F], [0.02843, 0.07819, 0.07819]),
    DARKTURQUOISE        = ([0x00, 0xCE, 0xD1], [0.0, 0.6172, 0.6376]),
    DARKVIOLET           = ([0x94, 0x00, 0xD3], [0.2961, 0.0, 0.6514]),
    DEEPPINK             = ([0xFF, 0x14, 0x93], [1.0, 0.006995, 0.2918]),
    DEEPSKYBLUE          = ([0x00, 0xBF, 0xFF], [0.0, 0.521, 1.0]),
    DIMGRAY              = ([0x69, 0x69, 0x69], [0.1413, 0.1413, 0.1413]),
    DODGERBLUE           = ([0x1E, 0x90, 0xFF], [0.01298, 0.2789, 1.0]),
    FIREBRICK            = ([0xB2, 0x22, 0x22], [0.4452, 0.016, 0.016]),
    FLORALWHITE          = ([0xFF, 0xFA, 0xF0], [1.0, 0.956, 0.8714]),
    FORESTGREEN          = ([0x22, 0x8B, 0x22], [0.016, 0.2582, 0.016]),
    FUCHSIA              = ([0xFF, 0x00, 0xFF], [1.0, 0.0, 1.0]),
    GAINSBORO            = ([0xDC, 0xDC, 0xDC], [0.7157, 0.7157, 0.7157]),
    GHOSTWHITE           = ([0xF8, 0xF8, 0xFF], [0.9387, 0.9387, 1.0]),
    GOLD                 = ([0xFF, 0xD7, 0x00], [1.0, 0.6795, 0.0]),
    GOLDENROD            = ([0xDA, 0xA5, 0x20], [0.7011, 0.3763, 0.01444]),
    GRAY                 = ([0x80, 0x80, 0x80], [0.2159, 0.2159, 0.2159]),
    GRAY0                = ([0x00, 0x00, 0x00], [0.0, 0.0, 0.0]),
    GRAY1                = ([0x11, 0x11, 0x11], [0.005605, 0.005605, 0.005605]),
    GRAY2                = ([0x22, 0x22, 0x22], [0.016, 0.016, 0.016]),
    GRAY3                = ([0x33, 0x33, 0x33], [0.0331, 0.0331, 0.0331]),
    GRAY4                = ([0x44, 0x44, 0x44], [0.05781, 0.05781, 0.05781]),
    GRAY5                = ([0x55, 0x55, 0x55], [0.09084, 0.09084, 0.09084]),
    GRAY6                = ([0x66, 0x66, 0x66], [0.1329, 0.1329, 0.1329]),
    GRAY7                = ([0x77, 0x77, 0x77], [0.1845, 0.1845, 0.1845]),
    GRAY8                = ([0x88, 0x88, 0x88], [0.2462, 0.2462, 0.2462]),
    GRAY9                = ([0x99, 0x99, 0x99], [0.3185, 0.3185, 0.3185]),
    GRAYA                = ([0xAA, 0xAA, 0xAA], [0.402, 0.402, 0.402]),
    GRAYB                = ([0xBB, 0xBB, 0xBB], [0.4969, 0.4969, 0.4969]),
    GRAYC                = ([0xCC, 0xCC, 0xCC], [0.6038, 0.6038, 0.6038]),
    GRAYD                = ([0xDD, 0xDD, 0xDD], [0.7231, 0.7231, 0.7231]),
    GRAYE                = ([0xEE, 0xEE, 0xEE], [0.855, 0.855, 0.855]),
    GRAYF                = ([0xFF, 0xFF, 0xFF], [1.0, 1.0, 1.0]),
    GREEN                = ([0x00, 0x80, 0x00], [0.0, 0.2159, 0.0]),
    GREENYELLOW          = ([0xAD, 0xFF, 0x2F], [0.4179, 1.0, 0.02843]),
    HONEYDEW             = ([0xF0, 0xFF, 0xF0], [0.8714, 1.0, 0.8714]),
    HOTPINK              = ([0xFF, 0x69, 0xB4], [1.0, 0.1413, 0.4564]),
    INDIANRED            = ([0xCD, 0x5C, 0x5C], [0.6105, 0.107, 0.107]),
    INDIGO               = ([0x4B, 0x00, 0x82], [0.07036, 0.0, 0.2232]),
    IVORY                = ([0xFF, 0xFF, 0xF0], [1.0, 1.0, 0.8714]),
    KHAKI                = ([0xF0, 0xE6, 0x8C], [0.8714, 0.7913, 0.2623]),
    LAVENDER             = ([0xE6, 0xE6, 0xFA], [0.7913, 0.7913, 0.956]),
    LAVENDERBLUSH        = ([0xFF, 0xF0, 0xF5], [1.0, 0.8714, 0.9131]),
    LAWNGREEN            = ([0x7C, 0xFC, 0x00], [0.2016, 0.9734, 0.0]),
    LEMONCHIFFON         = ([0xFF, 0xFA, 0xCD], [1.0, 0.956, 0.6105]),
    LIGHTBLUE            = ([0xAD, 0xD8, 0xE6], [0.4179, 0.6867, 0.7913]),
    LIGHTCORAL           = ([0xF0, 0x80, 0x80], [0.8714, 0.2159, 0.2159]),
    LIGHTCYAN            = ([0xE0, 0xFF, 0xFF], [0.7454, 1.0, 1.0]),
    LIGHTGOLDENRODYELLOW = ([0xFA, 0xFA, 0xD2], [0.956, 0.956, 0.6445]),
    LIGHTGREEN           = ([0x90, 0xEE, 0x90], [0.2789, 0.855, 0.2789]),
    LIGHTGRAY            = ([0xD3, 0xD3, 0xD3], [0.6514, 0.6514, 0.6514]),
    LIGHTPINK            = ([0xFF, 0xB6, 0xC1], [1.0, 0.4678, 0.5333]),
    LIGHTSALMON          = ([0xFF, 0xA0, 0x7A], [1.0, 0.3515, 0.1946]),
    LIGHTSEAGREEN        = ([0x20, 0xB2, 0xAA], [0.01444, 0.4452, 0.402]),
    LIGHTSKYBLUE         = ([0x87, 0xCE, 0xFA], [0.2423, 0.6172, 0.956]),
    LIGHTSLATEGRAY       = ([0x77, 0x88, 0x99], [0.1845, 0.2462, 0.3185]),
    LIGHTSTEELBLUE       = ([0xB0, 0xC4, 0xDE], [0.4342, 0.552, 0.7305]),
    LIGHTYELLOW          = ([0xFF, 0xFF, 0xE0], [1.0, 1.0, 0.7454]),
    LIME                 = ([0x00, 0xFF, 0x00], [0.0, 1.0, 0.0]),
    LIMEGREEN            = ([0x32, 0xCD, 0x32], [0.0319, 0.6105, 0.0319]),
    LINEN                = ([0xFA, 0xF0, 0xE6], [0.956, 0.8714, 0.7913]),
    MAGENTA              = ([0xFF, 0x00, 0xFF], [1.0, 0.0, 1.0]),
    MAROON               = ([0x80, 0x00, 0x00], [0.2159, 0.0, 0.0]),
    MEDIUMAQUAMARINE     = ([0x66, 0xCD, 0xAA], [0.1329, 0.6105, 0.402]),
    MEDIUMBLUE           = ([0x00, 0x00, 0xCD], [0.0, 0.0, 0.6105]),
    MEDIUMORCHID         = ([0xBA, 0x55, 0xD3], [0.491, 0.09084, 0.6514]),
    MEDIUMPURPLE         = ([0x93, 0x70, 0xDB], [0.2918, 0.162, 0.7084]),
    MEDIUMSEAGREEN       = ([0x3C, 0xB3, 0x71], [0.04519, 0.4508, 0.1651]),
    MEDIUMSLATEBLUE      = ([0x7B, 0x68, 0xEE], [0.1981, 0.1384, 0.855]),
    MEDIUMSPRINGGREEN    = ([0x00, 0xFA, 0x9A], [0.0, 0.956, 0.3231]),
    MEDIUMTURQUOISE      = ([0x48, 0xD1, 0xCC], [0.0648, 0.6376, 0.6038]),
    MEDIUMVIOLETRED      = ([0xC7, 0x15, 0x85], [0.5711, 0.007499, 0.2346]),
    MIDNIGHTBLUE         = ([0x19, 0x19, 0x70], [0.009721, 0.009721, 0.162]),
    MINTCREAM            = ([0xF5, 0xFF, 0xFA], [0.9131, 1.0, 0.956]),
    MISTYROSE            = ([0xFF, 0xE4, 0xE1], [1.0, 0.7758, 0.7529]),
    MOCCASIN             = ([0xFF, 0xE4, 0xB5], [1.0, 0.7758, 0.4621]),
    NAVAJOWHITE          = ([0xFF, 0xDE, 0xAD], [1.0, 0.7305, 0.4179]),
    NAVY                 = ([0x00, 0x00, 0x80], [0.0, 0.0, 0.2159]),
    OLDLACE              = ([0xFD, 0xF5, 0xE6], [0.9823, 0.9131, 0.7913]),
    OLIVE                = ([0x80, 0x80, 0x00], [0.2159, 0.2159, 0.0]),
    OLIVEDRAB            = ([0x6B, 0x8E, 0x23], [0.147, 0.2705, 0.01681]),
    ORANGE               = ([0xFF, 0xA5, 0x00], [1.0, 0.3763, 0.0]),
    ORANGERED            = ([0xFF, 0x45, 0x00], [1.0, 0.05951, 0.0]),
    ORCHID               = ([0xDA, 0x70, 0xD6], [0.7011, 0.162, 0.6724]),
    PALEGOLDENROD        = ([0xEE, 0xE8, 0xAA], [0.855, 0.807, 0.402]),
    PALEGREEN            = ([0x98, 0xFB, 0x98], [0.314, 0.9647, 0.314]),
    PALEVIOLETRED        = ([0xDB, 0x70, 0x93], [0.7084, 0.162, 0.2918]),
    PAPAYAWHIP           = ([0xFF, 0xEF, 0xD5], [1.0, 0.8632, 0.6654]),
    PEACHPUFF            = ([0xFF, 0xDA, 0xB9], [1.0, 0.7011, 0.4851]),
    PERU                 = ([0xCD, 0x85, 0x3F], [0.6105, 0.2346, 0.04971]),
    PINK                 = ([0xFF, 0xC0, 0xCB], [1.0, 0.5271, 0.5972]),
    PLUM                 = ([0xDD, 0xA0, 0xDD], [0.7231, 0.3515, 0.7231]),
    POWDERBLUE           = ([0xB0, 0xE0, 0xE6], [0.4342, 0.7454, 0.7913]),
    PURPLE               = ([0x80, 0x00, 0x80], [0.2159, 0.0, 0.2159]),
    RED                  = ([0xFF, 0x00, 0x00], [1.0, 0.0, 0.0]),
    ROSYBROWN            = ([0xBC, 0x8F, 0x8F], [0.5029, 0.2747, 0.2747]),
    ROYALBLUE            = ([0x41, 0x69, 0xE1], [0.05286, 0.1413, 0.7529]),
    SADDLEBROWN          = ([0x8B, 0x45, 0x13], [0.2582, 0.05951, 0.006512]),
    SALMON               = ([0xFA, 0x80, 0x72], [0.956, 0.2159, 0.1683]),
    SANDYBROWN           = ([0xFA, 0xA4, 0x60], [0.956, 0.3712, 0.117]),
    SEAGREEN             = ([0x2E, 0x8B, 0x57], [0.02732, 0.2582, 0.09531]),
    SEASHELL             = ([0xFF, 0xF5, 0xEE], [1.0, 0.9131, 0.855]),
    SIENNA               = ([0xA0, 0x52, 0x2D], [0.3515, 0.08438, 0.02624]),
    SILVER               = ([0xC0, 0xC0, 0xC0], [0.5271, 0.5271, 0.5271]),
    SKYBLUE              = ([0x87, 0xCE, 0xEB], [0.2423, 0.6172, 0.8308]),
    SLATEBLUE            = ([0x6A, 0x5A, 0xCD], [0.1441, 0.1022, 0.6105]),
    SLATEGRAY            = ([0x70, 0x80, 0x90], [0.162, 0.2159, 0.2789]),
    SNOW                 = ([0xFF, 0xFA, 0xFA], [1.0, 0.956, 0.956]),
    SPRINGGREEN          = ([0x00, 0xFF, 0x7F], [0.0, 1.0, 0.2122]),
    STEELBLUE            = ([0x46, 0x82, 0xB4], [0.06125, 0.2232, 0.4564]),
    TAN                  = ([0xD2, 0xB4, 0x8C], [0.6445, 0.4564, 0.2623]),
    TEAL                 = ([0x00, 0x80, 0x80], [0.0, 0.2159, 0.2159]),
    THISTLE              = ([0xD8, 0xBF, 0xD8], [0.6867, 0.521, 0.6867]),
    TOMATO               = ([0xFF, 0x63, 0x47], [1.0, 0.1248, 0.06301]),
    TURQUOISE            = ([0x40, 0xE0, 0xD0], [0.05127, 0.7454, 0.6308]),
    VIOLET               = ([0xEE, 0x82, 0xEE], [0.855, 0.2232, 0.855]),
    WHEAT                = ([0xF5, 0xDE, 0xB3], [0.9131, 0.7305, 0.4508]),
    WHITE                = ([0xFF, 0xFF, 0xFF], [1.0, 1.0, 1.0]),
    WHITESMOKE           = ([0xF5, 0xF5, 0xF5], [0.9131, 0.9131, 0.9131]),
    YELLOW               = ([0xFF, 0xFF, 0x00], [1.0, 1.0, 0.0]),
    YELLOWGREEN          = ([0x9A, 0xCD, 0x32], [0.3231, 0.6105, 0.0319]),
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
            } else {
                // Otherwise just do a hard switch at 50 %, can't gradient.
                if x < 0.5 {
                    PseudoTermColor::Solid(*self)
                } else {
                    PseudoTermColor::Solid(*other)
                }
            }
        } else {
            if x < 0.125 {
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
                fore: _,
                back: _,
                mix: ColorMix::Mix25,
            } => '░',
            PseudoTermColor::Mixed {
                fore: _,
                back: _,
                mix: ColorMix::Mix75,
            } => '▓',
            PseudoTermColor::Mixed {
                fore: _,
                back: _,
                mix: _,
            } => '▒',
            PseudoTermColor::Solid(_) => '█',
        }
    }

    /// Return the closest color to this pseudocolor.
    pub fn color(self) -> TermColor {
        match self {
            PseudoTermColor::Mixed {
                fore: _,
                back: base,
                mix: ColorMix::Mix25,
            } => TermColor {
                base,
                is_bright: false,
            },
            PseudoTermColor::Mixed {
                fore: _,
                back: base,
                mix: ColorMix::Mix50Low,
            } => TermColor {
                base,
                is_bright: false,
            },
            PseudoTermColor::Mixed {
                fore: c,
                back: _,
                mix: ColorMix::Mix50High,
            } => c,
            PseudoTermColor::Mixed {
                fore: c,
                back: _,
                mix: ColorMix::Mix75,
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
            return SRgba::rgb(192, 192, 192);
        } else if c.0 == 8 {
            return SRgba::rgb(128, 128, 128);
        } else if c.0 < 16 {
            let i = if c.0 & 0b1000 != 0 { 255 } else { 128 };
            let r = if c.0 & 0b1 != 0 { i } else { 0 };
            let g = if c.0 & 0b10 != 0 { i } else { 0 };
            let b = if c.0 & 0b100 != 0 { i } else { 0 };

            return SRgba::rgb(r * i, g * i, b * i);
        } else if c.0 < 232 {
            fn channel(i: u8) -> u8 { i * 40 + if i > 0 { 55 } else { 0 } }
            // 6^3 RGB space
            let c = c.0 - 16;
            let b = channel(c % 6);
            let g = channel((c / 6) % 6);
            let r = channel(c / 36);

            return SRgba::rgb(r, g, b);
        } else {
            // 24 level grayscale slide
            let c = c.0 - 232;
            let c = 8 + 10 * c;
            return SRgba::rgb(c, c, c);
        }
    }
}

impl From<SRgba> for Xterm256Color {
    fn from(c: SRgba) -> Xterm256Color {
        // Never convert into the first 16 colors, the user may have configured those to have
        // different RGB values.

        fn rgb_channel(c: u8) -> u8 {
            if c < 48 {
                return 0;
            } else if c < 75 {
                return 1;
            } else {
                return (c - 35) / 40;
            }
        }

        fn gray_channel(c: u8) -> u8 {
            if c < 13 {
                return 0;
            } else if c > 235 {
                return 23;
            } else {
                return (c - 3) / 10;
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

        if c.distance2(&rgb_color.into()) < c.distance2(&gray_color.into()) {
            rgb_color
        } else {
            gray_color
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse_color() {
        use super::{Rgba, SRgba};
        use std::str::FromStr;

        assert!(SRgba::from_str("").is_err());
        assert!(SRgba::from_str("#").is_err());
        assert!(SRgba::from_str("#12").is_err());
        assert!(SRgba::from_str("#123456789ABC").is_err());
        assert!(SRgba::from_str("#ff0000garbage").is_err());
        assert!(SRgba::from_str("#ffjunk").is_err());
        assert!(SRgba::from_str("actuallynotacolorname").is_err());
        assert!(SRgba::from_str("redd").is_err());

        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("#f00")
        );
        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("#f00f")
        );
        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("#ff0000")
        );
        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("#ff0000ff")
        );
        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("#FF0000FF")
        );
        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("red")
        );
        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("Red")
        );
        assert_eq!(
            Ok(SRgba::new(0xff, 0x00, 0x00, 0xff)),
            SRgba::from_str("RED")
        );
        assert_eq!(Ok(Rgba::new(1.0, 0.0, 0.0, 1.0)), Rgba::from_str("RED"));

        assert_eq!(SRgba::new(0x33, 0x77, 0xbb, 0xff), 0x3377bbff.into());

        assert_eq!(0x00, SRgba::from_str("#000").unwrap().r);
        assert_eq!(0x22, SRgba::from_str("#200").unwrap().r);
        assert_eq!(0xFF, SRgba::from_str("#F00").unwrap().r);
    }

    #[test]
    fn test_xterm256() {
        use super::{SRgba, Xterm256Color};
        for i in 16..256 {
            let c = Xterm256Color(i as u8);
            let rgb: SRgba = c.into();
            assert_eq!(c, Xterm256Color::from(rgb));
        }
    }

    #[test]
    fn test_luma() {
        // Test that luma values from corresponding sRGBA and RGBA are same.

        use super::{to_srgb, Rgba, SRgba};
        use rand::{Rng, SeedableRng, XorShiftRng};

        let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);
        for _ in 0..1000 {
            let sc = SRgba::rgb(rng.gen(), rng.gen(), rng.gen());
            let sluma = sc.luma();

            let c = Rgba::from(sc);
            let cluma = c.luma();
            let csluma = (to_srgb(cluma) * 255.0).round() as u8;

            assert_eq!(sluma, csluma);
            assert_eq!(sc, SRgba::from(c));
        }
    }
}
