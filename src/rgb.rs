use std::num::{from_str_radix};
use std::ascii::{OwnedAsciiExt};
use color;
use ::Color;

#[derive(Copy, Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r: r, g: g, b: b }
    }

    /// Panics if name doesn't parse, use for inline constants.
    pub fn parse(name: &str) -> Rgb {
        let rgba = Rgba::parse(name);
        Rgb::new(rgba.r, rgba.g, rgba.b)
    }
}

impl Color for Rgb {
    fn to_rgba(&self) -> [f32; 4] {
        [self.r as f32 / 255.0,
         self.g as f32 / 255.0,
         self.b as f32 / 255.0,
         1.0]
    }

    fn from_color<C: Color>(color: &C) -> Rgb {
        let rgba = color.to_rgba();
        Rgb { r: (rgba[0] * 255.0) as u8, g: (rgba[1] * 255.0) as u8, b: (rgba[2] * 255.0) as u8 }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct Rgba { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r: r, g: g, b: b, a: a }
    }

    /// Panics if name doesn't parse, use for inline constants. Accept
    /// case-insensitive SVG color names ("red", "powderblue") and hex #RGB or
    /// #RGBA color names with 4 or 8 bits per channel. "#F00", "#F00F",
    /// "#FF0000" and "#FF0000FF" all correspond to the same opaque pure
    /// red color.
    pub fn parse(name: &str) -> Rgba {
        parse_color(name).expect(&format!("Invalid color name {}", name)[..])
    }
}

impl Color for Rgba {
    fn to_rgba(&self) -> [f32; 4] {
        [self.r as f32 / 255.0,
         self.g as f32 / 255.0,
         self.b as f32 / 255.0,
         self.a as f32 / 255.0]
    }

    fn from_color<C: Color>(color: &C) -> Rgba {
        let rgba = color.to_rgba();
        Rgba { r: (rgba[0] * 255.0) as u8, g: (rgba[1] * 255.0) as u8, b: (rgba[2] * 255.0) as u8, a: (rgba[3] * 255.0) as u8 }
    }
}

fn parse_color(name: &str) -> Option<Rgba> {
    if let Some(Rgb { r, g, b }) = parse_color_name(&name.to_string().into_ascii_lowercase()[..]) {
        return Some(Rgba { r: r, g: g, b: b, a: 0xFF });
    }

    let rgb = regex!(r"^#([0-9]|[a-f]|[A-F])([0-9]|[a-f]|[A-F])([0-9]|[a-f]|[A-F])$");
    if let Some(cap) = rgb.captures(name) {
        let r = from_str_radix::<u8>(cap.at(1).unwrap(), 16).unwrap();
        let g = from_str_radix::<u8>(cap.at(2).unwrap(), 16).unwrap();
        let b = from_str_radix::<u8>(cap.at(3).unwrap(), 16).unwrap();
        return Some(Rgba { r: (r << 4) + r, g: (g << 4) + g, b: (b << 4) + b, a: 0xFF });
    }

    let rgba = regex!(r"^#([0-9]|[a-f]|[A-F])([0-9]|[a-f]|[A-F])([0-9]|[a-f]|[A-F])([0-9]|[a-f]|[A-F])$");
    if let Some(cap) = rgba.captures(name) {
        let r = from_str_radix::<u8>(cap.at(1).unwrap(), 16).unwrap();
        let g = from_str_radix::<u8>(cap.at(2).unwrap(), 16).unwrap();
        let b = from_str_radix::<u8>(cap.at(3).unwrap(), 16).unwrap();
        let a = from_str_radix::<u8>(cap.at(4).unwrap(), 16).unwrap();
        return Some(Rgba { r: (r << 4) + r, g: (g << 4) + g, b: (b << 4) + b, a: (a << 4) + a });
    }

    let rrggbb = regex!(r"^#((?:[0-9]|[a-f]|[A-F]){2})((?:[0-9]|[a-f]|[A-F]){2})((?:[0-9]|[a-f]|[A-F]){2})$");
    if let Some(cap) = rrggbb.captures(name) {
        let r = from_str_radix::<u8>(cap.at(1).unwrap(), 16).unwrap();
        let g = from_str_radix::<u8>(cap.at(2).unwrap(), 16).unwrap();
        let b = from_str_radix::<u8>(cap.at(3).unwrap(), 16).unwrap();
        return Some(Rgba { r: r, g: g, b: b, a: 0xFF });
    }

    let rrggbbaa = regex!(r"^#((?:[0-9]|[a-f]|[A-F]){2})((?:[0-9]|[a-f]|[A-F]){2})((?:[0-9]|[a-f]|[A-F]){2})((?:[0-9]|[a-f]|[A-F]){2})$");
    if let Some(cap) = rrggbbaa.captures(name) {
        let r = from_str_radix::<u8>(cap.at(1).unwrap(), 16).unwrap();
        let g = from_str_radix::<u8>(cap.at(2).unwrap(), 16).unwrap();
        let b = from_str_radix::<u8>(cap.at(3).unwrap(), 16).unwrap();
        let a = from_str_radix::<u8>(cap.at(4).unwrap(), 16).unwrap();
        return Some(Rgba { r: r, g: g, b: b, a: a });
    }

    return None;
}

fn parse_color_name(lower_case_name: &str) -> Option<Rgb> {
    match lower_case_name {
        "aliceblue" => Some(color::ALICEBLUE),
        "antiquewhite" => Some(color::ANTIQUEWHITE),
        "aqua" => Some(color::AQUA),
        "aquamarine" => Some(color::AQUAMARINE),
        "azure" => Some(color::AZURE),
        "beige" => Some(color::BEIGE),
        "bisque" => Some(color::BISQUE),
        "black" => Some(color::BLACK),
        "blanchedalmond" => Some(color::BLANCHEDALMOND),
        "blue" => Some(color::BLUE),
        "blueviolet" => Some(color::BLUEVIOLET),
        "brown" => Some(color::BROWN),
        "burlywood" => Some(color::BURLYWOOD),
        "cadetblue" => Some(color::CADETBLUE),
        "chartreuse" => Some(color::CHARTREUSE),
        "chocolate" => Some(color::CHOCOLATE),
        "coral" => Some(color::CORAL),
        "cornflowerblue" => Some(color::CORNFLOWERBLUE),
        "cornsilk" => Some(color::CORNSILK),
        "crimson" => Some(color::CRIMSON),
        "cyan" => Some(color::CYAN),
        "darkblue" => Some(color::DARKBLUE),
        "darkcyan" => Some(color::DARKCYAN),
        "darkgoldenrod" => Some(color::DARKGOLDENROD),
        "darkgray" => Some(color::DARKGRAY),
        "darkgreen" => Some(color::DARKGREEN),
        "darkkhaki" => Some(color::DARKKHAKI),
        "darkmagenta" => Some(color::DARKMAGENTA),
        "darkolivegreen" => Some(color::DARKOLIVEGREEN),
        "darkorange" => Some(color::DARKORANGE),
        "darkorchid" => Some(color::DARKORCHID),
        "darkred" => Some(color::DARKRED),
        "darksalmon" => Some(color::DARKSALMON),
        "darkseagreen" => Some(color::DARKSEAGREEN),
        "darkslateblue" => Some(color::DARKSLATEBLUE),
        "darkslategray" => Some(color::DARKSLATEGRAY),
        "darkturquoise" => Some(color::DARKTURQUOISE),
        "darkviolet" => Some(color::DARKVIOLET),
        "deeppink" => Some(color::DEEPPINK),
        "deepskyblue" => Some(color::DEEPSKYBLUE),
        "dimgray" => Some(color::DIMGRAY),
        "dodgerblue" => Some(color::DODGERBLUE),
        "firebrick" => Some(color::FIREBRICK),
        "floralwhite" => Some(color::FLORALWHITE),
        "forestgreen" => Some(color::FORESTGREEN),
        "fuchsia" => Some(color::FUCHSIA),
        "gainsboro" => Some(color::GAINSBORO),
        "ghostwhite" => Some(color::GHOSTWHITE),
        "gold" => Some(color::GOLD),
        "goldenrod" => Some(color::GOLDENROD),
        "gray" => Some(color::GRAY),
        "green" => Some(color::GREEN),
        "greenyellow" => Some(color::GREENYELLOW),
        "honeydew" => Some(color::HONEYDEW),
        "hotpink" => Some(color::HOTPINK),
        "indianred" => Some(color::INDIANRED),
        "indigo" => Some(color::INDIGO),
        "ivory" => Some(color::IVORY),
        "khaki" => Some(color::KHAKI),
        "lavender" => Some(color::LAVENDER),
        "lavenderblush" => Some(color::LAVENDERBLUSH),
        "lawngreen" => Some(color::LAWNGREEN),
        "lemonchiffon" => Some(color::LEMONCHIFFON),
        "lightblue" => Some(color::LIGHTBLUE),
        "lightcoral" => Some(color::LIGHTCORAL),
        "lightcyan" => Some(color::LIGHTCYAN),
        "lightgoldenrodyellow" => Some(color::LIGHTGOLDENRODYELLOW),
        "lightgreen" => Some(color::LIGHTGREEN),
        "lightgray" => Some(color::LIGHTGRAY),
        "lightpink" => Some(color::LIGHTPINK),
        "lightsalmon" => Some(color::LIGHTSALMON),
        "lightseagreen" => Some(color::LIGHTSEAGREEN),
        "lightskyblue" => Some(color::LIGHTSKYBLUE),
        "lightslategray" => Some(color::LIGHTSLATEGRAY),
        "lightsteelblue" => Some(color::LIGHTSTEELBLUE),
        "lightyellow" => Some(color::LIGHTYELLOW),
        "lime" => Some(color::LIME),
        "limegreen" => Some(color::LIMEGREEN),
        "linen" => Some(color::LINEN),
        "magenta" => Some(color::MAGENTA),
        "maroon" => Some(color::MAROON),
        "mediumaquamarine" => Some(color::MEDIUMAQUAMARINE),
        "mediumblue" => Some(color::MEDIUMBLUE),
        "mediumorchid" => Some(color::MEDIUMORCHID),
        "mediumpurple" => Some(color::MEDIUMPURPLE),
        "mediumseagreen" => Some(color::MEDIUMSEAGREEN),
        "mediumslateblue" => Some(color::MEDIUMSLATEBLUE),
        "mediumspringgreen" => Some(color::MEDIUMSPRINGGREEN),
        "mediumturquoise" => Some(color::MEDIUMTURQUOISE),
        "mediumvioletred" => Some(color::MEDIUMVIOLETRED),
        "midnightblue" => Some(color::MIDNIGHTBLUE),
        "mintcream" => Some(color::MINTCREAM),
        "mistyrose" => Some(color::MISTYROSE),
        "moccasin" => Some(color::MOCCASIN),
        "navajowhite" => Some(color::NAVAJOWHITE),
        "navy" => Some(color::NAVY),
        "oldlace" => Some(color::OLDLACE),
        "olive" => Some(color::OLIVE),
        "olivedrab" => Some(color::OLIVEDRAB),
        "orange" => Some(color::ORANGE),
        "orangered" => Some(color::ORANGERED),
        "orchid" => Some(color::ORCHID),
        "palegoldenrod" => Some(color::PALEGOLDENROD),
        "palegreen" => Some(color::PALEGREEN),
        "palevioletred" => Some(color::PALEVIOLETRED),
        "papayawhip" => Some(color::PAPAYAWHIP),
        "peachpuff" => Some(color::PEACHPUFF),
        "peru" => Some(color::PERU),
        "pink" => Some(color::PINK),
        "plum" => Some(color::PLUM),
        "powderblue" => Some(color::POWDERBLUE),
        "purple" => Some(color::PURPLE),
        "red" => Some(color::RED),
        "rosybrown" => Some(color::ROSYBROWN),
        "royalblue" => Some(color::ROYALBLUE),
        "saddlebrown" => Some(color::SADDLEBROWN),
        "salmon" => Some(color::SALMON),
        "sandybrown" => Some(color::SANDYBROWN),
        "seagreen" => Some(color::SEAGREEN),
        "seashell" => Some(color::SEASHELL),
        "sienna" => Some(color::SIENNA),
        "silver" => Some(color::SILVER),
        "skyblue" => Some(color::SKYBLUE),
        "slateblue" => Some(color::SLATEBLUE),
        "slategray" => Some(color::SLATEGRAY),
        "snow" => Some(color::SNOW),
        "springgreen" => Some(color::SPRINGGREEN),
        "steelblue" => Some(color::STEELBLUE),
        "tan" => Some(color::TAN),
        "teal" => Some(color::TEAL),
        "thistle" => Some(color::THISTLE),
        "tomato" => Some(color::TOMATO),
        "turquoise" => Some(color::TURQUOISE),
        "violet" => Some(color::VIOLET),
        "wheat" => Some(color::WHEAT),
        "white" => Some(color::WHITE),
        "whitesmoke" => Some(color::WHITESMOKE),
        "yellow" => Some(color::YELLOW),
        "yellowgreen" => Some(color::YELLOWGREEN),
        _ => None
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse_color() {
        use super::Rgba;
        use super::parse_color;

        assert_eq!(None, parse_color(""));
        assert_eq!(None, parse_color("#"));
        assert_eq!(None, parse_color("#12"));
        assert_eq!(None, parse_color("#123456789ABC"));
        assert_eq!(None, parse_color("#ff0000garbage"));
        assert_eq!(None, parse_color("#ffjunk"));
        assert_eq!(None, parse_color("actuallynotacolorname"));
        assert_eq!(None, parse_color("redd"));

        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("#f00"));
        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("#f00f"));
        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("#ff0000"));
        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("#ff0000ff"));
        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("#FF0000FF"));
        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("red"));
        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("Red"));
        assert_eq!(Some(Rgba { r: 0xFF, g: 0x00, b: 0x00, a: 0xFF }), parse_color("RED"));
    }
}
