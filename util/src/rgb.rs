#[derive(Copy, Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r: r, g: g, b: b }
    }
}

impl ::Color for Rgb {
    fn to_rgba(&self) -> [f32; 4] {
        [self.r as f32 / 255.0,
         self.g as f32 / 255.0,
         self.b as f32 / 255.0,
         1.0]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub struct Rgba { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r: r, g: g, b: b, a: a }
    }
}

impl ::Color for Rgba {
    fn to_rgba(&self) -> [f32; 4] {
        [self.r as f32 / 255.0,
         self.g as f32 / 255.0,
         self.b as f32 / 255.0,
         self.a as f32 / 255.0]
    }
}
