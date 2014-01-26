#[crate_id = "stb"];
#[desc = "Static STB library integration"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[feature(link_args)];
#[feature(globs)];

extern "C" {
#[link_args="src/stb/stb_truetype.c"];
#[link_args="src/stb/stb_image.c"];
#[link_args="-fPIC"];
}

pub mod image;
pub mod truetype;
