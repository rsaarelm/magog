#![crate_name="magog"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]

extern crate num;
extern crate rand;
extern crate time;
extern crate cgmath;
extern crate image;
extern crate calx;
extern crate uuid;
extern crate world;

pub mod view {
    pub mod drawable;
    //pub mod worldview;
    pub mod tilecache;
    pub mod main;
    //pub mod titlestate;
    //pub mod gamestate;
}

pub fn main() {
    view::main::main()
}
