#[desc = "Static STB library integration"];
#[license = "MIT"];

#[feature(link_args)];
#[feature(globs)];

extern "C" {
#[link_args="src/stb/stb_truetype.c"];
#[link_args="src/stb/stb_image.c"];
#[link_args="-fPIC"];
}

pub mod image;
pub mod truetype;
