extern crate blot;

use blot::window;

fn main() {
    println!("Hello, world!");
    for evt in window::Window::new().run() {
    }
}
