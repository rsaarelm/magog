#[desc = "Shared gamelib"];
#[license = "GPLv3"];
#[feature(globs)];

extern mod extra;

pub mod text;

pub fn hello() {
    println!("Hello, world!");
}
