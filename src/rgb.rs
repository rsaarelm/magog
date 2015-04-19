use std::num::{Float};
use std::cell::RefCell;
use std::collections::HashMap;
use std::num::{from_str_radix};
use std::ascii::{OwnedAsciiExt};
use std::ops::{Add, Sub, Mul};

/// Things that describe a color.
pub trait ToColor {
    /// Convert a color to linear RGBA values you can feed to OpenGL.
    fn to_rgba(&self) -> [f32; 4];

    /// Convert a color to sRGBA, the byte values you actually get on your
    /// screen.
    fn to_srgba(&self) -> [u8; 4] {
        let rgba = self.to_rgba();
        [(to_srgb(rgba[0]) * 255.0).round() as u8,
         (to_srgb(rgba[1]) * 255.0).round() as u8,
         (to_srgb(rgba[2]) * 255.0).round() as u8,
         (to_srgb(rgba[3]) * 255.0).round() as u8]
    }
}

/// Things that can be made from a color.
pub trait FromColor: Sized {
    /// Build the value from linear color components.
    fn from_rgba(rgba: [f32; 4]) -> Self;

    /// Build the value from sRGBA color components.
    fn from_srgba(srgba: [u8; 4]) -> Self {
        FromColor::from_rgba([
            to_linear(srgba[0] as f32 / 255.0),
            to_linear(srgba[1] as f32 / 255.0),
            to_linear(srgba[2] as f32 / 255.0),
            to_linear(srgba[3] as f32 / 255.0)])
    }

    fn from_color<C: ToColor>(color: &C) -> Self {
        FromColor::from_rgba(color.to_rgba())
    }
}

/// Color in linear color space.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, RustcEncodable, RustcDecodable)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba { r: r, g: g, b: b, a: a }
    }
}

impl ToColor for Rgba {
    fn to_rgba(&self) -> [f32; 4] { [self.r, self.g, self.b, self.a] }
}

impl FromColor for Rgba {
    fn from_rgba(rgba: [f32; 4]) -> Rgba {
        Rgba { r: rgba[0], g: rgba[1], b: rgba[2], a: rgba[3] }
    }
}

impl Add<Rgba> for Rgba {
    type Output = Rgba;
    fn add(self, rhs: Rgba) -> Rgba {
        Rgba::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a)
    }
}

impl Sub<Rgba> for Rgba {
    type Output = Rgba;
    fn sub(self, rhs: Rgba) -> Rgba {
        Rgba::new(
            self.r - rhs.r,
            self.g - rhs.g,
            self.b - rhs.b,
            self.a - rhs.a)
    }
}

impl Mul<f32> for Rgba {
    type Output = Rgba;
    fn mul(self, rhs: f32) -> Rgba {
        Rgba::new(
            self.r * rhs,
            self.g * rhs,
            self.b * rhs,
            self.a * rhs)
    }
}

impl Mul<Rgba> for Rgba {
    type Output = Rgba;
    fn mul(self, rhs: Rgba) -> Rgba {
        Rgba::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a)
    }
}

pub fn to_linear(srgb: f32) -> f32 {
   if srgb <= 0.04045 {
       srgb / 12.92
   } else {
       ((srgb + 0.055) / (1.055)).powf(2.4)
   }
}

pub fn to_srgb(linear: f32) -> f32 {
    if linear < 0.0031308 {
        12.92 * linear
    } else {
        (1.0 + 0.055) * linear.powf(1.0 / 2.4) - 0.055
    }
}


/// Neat trick for using plain string literals as color values. Will panic if
/// given non-parsing color.
///
/// Accepts case-insensitive SVG color names ("red", "powderblue") and hex
/// #RGB or #RGBA color names with 4 or 8 bits per channel. "#F00", "#F00F",
/// "#FF0000" and "#FF0000FF" all correspond to the same opaque pure red
/// color.
impl ToColor for &'static str {
    fn to_rgba(&self) -> [f32; 4] {
        thread_local!(static MEMOIZER: RefCell<HashMap<String, [f32; 4]>> =
                      RefCell::new(HashMap::new()));

        let ret = MEMOIZER.with(|c| c.borrow().get(*self).map(|&x| x));
        match ret {
            Some(color) => color,
            None => {
                let parsed = parse_color(self)
                    .expect(&format!("Bad color string '{}'", self))
                    .to_rgba();
                MEMOIZER.with(|c| c.borrow_mut().insert(self.to_string(), parsed));
                parsed
            }
        }
    }
}

fn parse_color(name: &str) -> Option<Rgba> {
    if let Some(color) = parse_color_name(&name.to_string().into_ascii_uppercase()[..]) {
        return Some(color);
    }

    if name.starts_with("#") {
        let name = &name[1..];

        // Hex digits per color channel, either 1 or 2. Single digit values
        // get doubled for the color, #420 becomes #442200.
        let digits: usize;

        // Does the color include the alpha channel. If not, assume alpha is
        // fully opaque.
        let alpha: bool;

        match name.len() {
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
            _ => { return None; }
        }

        assert!(digits == 1 || digits == 2);

        let r = u8::from_str_radix(&name[0..(digits)], 16);
        let g = u8::from_str_radix(&name[(digits)..(2 * digits)], 16);
        let b = u8::from_str_radix(&name[(2 * digits)..(3 * digits)], 16);
        let a = if alpha {
            u8::from_str_radix(&name[(3 * digits)..(4 * digits)], 16)
        } else {
            if digits == 1 { Ok(0xFu8) } else { Ok(0xFFu8) }
        };

        return match (r, g, b, a) {
            (Ok(mut r), Ok(mut g), Ok(mut b), Ok(mut a)) => {
                if digits == 1 {
                    r = (r << 4) + r;
                    g = (g << 4) + g;
                    b = (b << 4) + b;
                    a = (a << 4) + a;
                }

                Some(FromColor::from_srgba([r, g, b, a]))
            }
            _ => None
        };
    }

    return None;
}

macro_rules! color_constants {
    {
        $([$name:ident, $r:expr, $g:expr, $b:expr])+
    } => {
        pub mod color {
            /*! Color constants */
            use super::Rgba;
            $(pub static $name: Rgba = Rgba { r: $r, g: $g, b: $b, a: 1.0 };)+
        }

        fn parse_color_name(upper_case_name: &str) -> Option<Rgba> {
            match upper_case_name {
                $(stringify!($name) => Some(color::$name),)+
                _ => None
            }
        }
    }
}

color_constants!{
[ALICEBLUE, 0.8714, 0.9387, 1.0]
[ANTIQUEWHITE, 0.956, 0.8308, 0.6795]
[AQUA, 0.0, 1.0, 1.0]
[AQUAMARINE, 0.2122, 1.0, 0.6584]
[AZURE, 0.8714, 1.0, 1.0]
[BEIGE, 0.9131, 0.9131, 0.7157]
[BISQUE, 1.0, 0.7758, 0.552]
[BLACK, 0.0, 0.0, 0.0]
[BLANCHEDALMOND, 1.0, 0.8308, 0.6105]
[BLUE, 0.0, 0.0, 1.0]
[BLUEVIOLET, 0.2542, 0.02416, 0.7605]
[BROWN, 0.3763, 0.02315, 0.02315]
[BURLYWOOD, 0.7305, 0.4793, 0.2423]
[CADETBLUE, 0.1144, 0.3419, 0.3515]
[CHARTREUSE, 0.2122, 1.0, 0.0]
[CHOCOLATE, 0.6445, 0.1413, 0.01298]
[CORAL, 1.0, 0.2122, 0.08022]
[CORNFLOWERBLUE, 0.1274, 0.3005, 0.8469]
[CORNSILK, 1.0, 0.9387, 0.7157]
[CRIMSON, 0.7157, 0.006995, 0.04519]
[CYAN, 0.0, 1.0, 1.0]
[DARKBLUE, 0.0, 0.0, 0.2582]
[DARKCYAN, 0.0, 0.2582, 0.2582]
[DARKGOLDENROD, 0.4793, 0.2384, 0.003347]
[DARKGRAY, 0.3968, 0.3968, 0.3968]
[DARKGREEN, 0.0, 0.1274, 0.0]
[DARKKHAKI, 0.5089, 0.4735, 0.147]
[DARKMAGENTA, 0.2582, 0.0, 0.2582]
[DARKOLIVEGREEN, 0.09084, 0.147, 0.02843]
[DARKORANGE, 1.0, 0.2623, 0.0]
[DARKORCHID, 0.3185, 0.0319, 0.6038]
[DARKRED, 0.2582, 0.0, 0.0]
[DARKSALMON, 0.8148, 0.305, 0.1946]
[DARKSEAGREEN, 0.2747, 0.5029, 0.2747]
[DARKSLATEBLUE, 0.0648, 0.04667, 0.2582]
[DARKSLATEGRAY, 0.02843, 0.07819, 0.07819]
[DARKTURQUOISE, 0.0, 0.6172, 0.6376]
[DARKVIOLET, 0.2961, 0.0, 0.6514]
[DEEPPINK, 1.0, 0.006995, 0.2918]
[DEEPSKYBLUE, 0.0, 0.521, 1.0]
[DIMGRAY, 0.1413, 0.1413, 0.1413]
[DODGERBLUE, 0.01298, 0.2789, 1.0]
[FIREBRICK, 0.4452, 0.016, 0.016]
[FLORALWHITE, 1.0, 0.956, 0.8714]
[FORESTGREEN, 0.016, 0.2582, 0.016]
[FUCHSIA, 1.0, 0.0, 1.0]
[GAINSBORO, 0.7157, 0.7157, 0.7157]
[GHOSTWHITE, 0.9387, 0.9387, 1.0]
[GOLD, 1.0, 0.6795, 0.0]
[GOLDENROD, 0.7011, 0.3763, 0.01444]
[GRAY, 0.2159, 0.2159, 0.2159]
[GREEN, 0.0, 0.2159, 0.0]
[GREENYELLOW, 0.4179, 1.0, 0.02843]
[HONEYDEW, 0.8714, 1.0, 0.8714]
[HOTPINK, 1.0, 0.1413, 0.4564]
[INDIANRED, 0.6105, 0.107, 0.107]
[INDIGO, 0.07036, 0.0, 0.2232]
[IVORY, 1.0, 1.0, 0.8714]
[KHAKI, 0.8714, 0.7913, 0.2623]
[LAVENDER, 0.7913, 0.7913, 0.956]
[LAVENDERBLUSH, 1.0, 0.8714, 0.9131]
[LAWNGREEN, 0.2016, 0.9734, 0.0]
[LEMONCHIFFON, 1.0, 0.956, 0.6105]
[LIGHTBLUE, 0.4179, 0.6867, 0.7913]
[LIGHTCORAL, 0.8714, 0.2159, 0.2159]
[LIGHTCYAN, 0.7454, 1.0, 1.0]
[LIGHTGOLDENRODYELLOW, 0.956, 0.956, 0.6445]
[LIGHTGREEN, 0.2789, 0.855, 0.2789]
[LIGHTGRAY, 0.6514, 0.6514, 0.6514]
[LIGHTPINK, 1.0, 0.4678, 0.5333]
[LIGHTSALMON, 1.0, 0.3515, 0.1946]
[LIGHTSEAGREEN, 0.01444, 0.4452, 0.402]
[LIGHTSKYBLUE, 0.2423, 0.6172, 0.956]
[LIGHTSLATEGRAY, 0.1845, 0.2462, 0.3185]
[LIGHTSTEELBLUE, 0.4342, 0.552, 0.7305]
[LIGHTYELLOW, 1.0, 1.0, 0.7454]
[LIME, 0.0, 1.0, 0.0]
[LIMEGREEN, 0.0319, 0.6105, 0.0319]
[LINEN, 0.956, 0.8714, 0.7913]
[MAGENTA, 1.0, 0.0, 1.0]
[MAROON, 0.2159, 0.0, 0.0]
[MEDIUMAQUAMARINE, 0.1329, 0.6105, 0.402]
[MEDIUMBLUE, 0.0, 0.0, 0.6105]
[MEDIUMORCHID, 0.491, 0.09084, 0.6514]
[MEDIUMPURPLE, 0.2918, 0.162, 0.7084]
[MEDIUMSEAGREEN, 0.04519, 0.4508, 0.1651]
[MEDIUMSLATEBLUE, 0.1981, 0.1384, 0.855]
[MEDIUMSPRINGGREEN, 0.0, 0.956, 0.3231]
[MEDIUMTURQUOISE, 0.0648, 0.6376, 0.6038]
[MEDIUMVIOLETRED, 0.5711, 0.007499, 0.2346]
[MIDNIGHTBLUE, 0.009721, 0.009721, 0.162]
[MINTCREAM, 0.9131, 1.0, 0.956]
[MISTYROSE, 1.0, 0.7758, 0.7529]
[MOCCASIN, 1.0, 0.7758, 0.4621]
[NAVAJOWHITE, 1.0, 0.7305, 0.4179]
[NAVY, 0.0, 0.0, 0.2159]
[OLDLACE, 0.9823, 0.9131, 0.7913]
[OLIVE, 0.2159, 0.2159, 0.0]
[OLIVEDRAB, 0.147, 0.2705, 0.01681]
[ORANGE, 1.0, 0.3763, 0.0]
[ORANGERED, 1.0, 0.05951, 0.0]
[ORCHID, 0.7011, 0.162, 0.6724]
[PALEGOLDENROD, 0.855, 0.807, 0.402]
[PALEGREEN, 0.314, 0.9647, 0.314]
[PALEVIOLETRED, 0.7084, 0.162, 0.2918]
[PAPAYAWHIP, 1.0, 0.8632, 0.6654]
[PEACHPUFF, 1.0, 0.7011, 0.4851]
[PERU, 0.6105, 0.2346, 0.04971]
[PINK, 1.0, 0.5271, 0.5972]
[PLUM, 0.7231, 0.3515, 0.7231]
[POWDERBLUE, 0.4342, 0.7454, 0.7913]
[PURPLE, 0.2159, 0.0, 0.2159]
[RED, 1.0, 0.0, 0.0]
[ROSYBROWN, 0.5029, 0.2747, 0.2747]
[ROYALBLUE, 0.05286, 0.1413, 0.7529]
[SADDLEBROWN, 0.2582, 0.05951, 0.006512]
[SALMON, 0.956, 0.2159, 0.1683]
[SANDYBROWN, 0.956, 0.3712, 0.117]
[SEAGREEN, 0.02732, 0.2582, 0.09531]
[SEASHELL, 1.0, 0.9131, 0.855]
[SIENNA, 0.3515, 0.08438, 0.02624]
[SILVER, 0.5271, 0.5271, 0.5271]
[SKYBLUE, 0.2423, 0.6172, 0.8308]
[SLATEBLUE, 0.1441, 0.1022, 0.6105]
[SLATEGRAY, 0.162, 0.2159, 0.2789]
[SNOW, 1.0, 0.956, 0.956]
[SPRINGGREEN, 0.0, 1.0, 0.2122]
[STEELBLUE, 0.06125, 0.2232, 0.4564]
[TAN, 0.6445, 0.4564, 0.2623]
[TEAL, 0.0, 0.2159, 0.2159]
[THISTLE, 0.6867, 0.521, 0.6867]
[TOMATO, 1.0, 0.1248, 0.06301]
[TURQUOISE, 0.05127, 0.7454, 0.6308]
[VIOLET, 0.855, 0.2232, 0.855]
[WHEAT, 0.9131, 0.7305, 0.4508]
[WHITE, 1.0, 1.0, 1.0]
[WHITESMOKE, 0.9131, 0.9131, 0.9131]
[YELLOW, 1.0, 1.0, 0.0]
[YELLOWGREEN, 0.3231, 0.6105, 0.0319]
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse_color() {
        use super::{Rgba, parse_color, FromColor, ToColor};

        assert_eq!(None, parse_color(""));
        assert_eq!(None, parse_color("#"));
        assert_eq!(None, parse_color("#12"));
        assert_eq!(None, parse_color("#123456789ABC"));
        assert_eq!(None, parse_color("#ff0000garbage"));
        assert_eq!(None, parse_color("#ffjunk"));
        assert_eq!(None, parse_color("actuallynotacolorname"));
        assert_eq!(None, parse_color("redd"));

        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("#f00"));
        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("#f00f"));
        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("#ff0000"));
        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("#ff0000ff"));
        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("#FF0000FF"));
        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("red"));
        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("Red"));
        assert_eq!(Some(Rgba::new(1.0, 0.0, 0.0, 1.0)), parse_color("RED"));

        let c: Rgba = FromColor::from_color(&"#000");
        assert_eq!(0x00, c.to_srgba()[0]);
        let c: Rgba = FromColor::from_color(&"#200");
        assert_eq!(0x22, c.to_srgba()[0]);
        let c: Rgba = FromColor::from_color(&"#F00");
        assert_eq!(0xFF, c.to_srgba()[0]);
    }
}
