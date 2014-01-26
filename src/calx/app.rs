extern mod sdl2;

use std::vec;

use sdl2::pixels;
use sdl2::render;
use sdl2::video;

pub struct App {
    priv renderer: ~render::Renderer,
    priv texture: ~render::Texture,

    pixels: ~[u8],
    priv width: uint,
    priv height: uint,
}

impl App {
    pub fn new(title: ~str, width: uint, height: uint) -> Result<~App, ~str> {
        sdl2::init([sdl2::InitVideo]);

        let window = match video::Window::new(title,
            sdl2::video::PosCentered, sdl2::video::PosCentered,
            width as int, height as int, [video::OpenGL]) {
            Ok(window) => window,
            Err(err) => return Err(format!("SDL2 window fail: {}", err))
        };
        let renderer = match render::Renderer::from_window(window,
            render::DriverAuto, [render::Accelerated]) {
            Ok(renderer) => renderer,
            Err(err) => return Err(format!("SDL2 renderer fail: {}", err))
        };
        let texture = match renderer.create_texture(pixels::ARGB8888,
            render::AccessStreaming, width as int, height as int) {
            Ok(texture) => texture,
            Err(err) => return Err(format!("SDL2 texture fail: {}", err))
        };
        let pixels = vec::from_elem(width * height * 4, 0u8);
        Ok(~App{
            renderer: renderer,
            texture: texture,
            pixels: pixels,
            width: width,
            height: height,
        })
    }

    pub fn render(&self) {
        self.renderer.clear();
        self.texture.update(None, self.pixels, self.width as int * 4);
        self.renderer.copy(self.texture, None, None);
        self.renderer.present();
    }
}

