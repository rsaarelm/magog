extern mod sdl2;

use std::vec;

use sdl2::pixels;
use sdl2::render;
use sdl2::video;
use sdl2::rect::Rect;

pub struct App {
    priv renderer: ~render::Renderer,
    priv texture: ~render::Texture,
    priv width: uint,
    priv height: uint,

    // Screen buffer to write to.
    pixels: ~[u8],

    // When true, making the window bigger will only make the pixels grow in
    // integer multiples. Keeps pixel art pretty.
    pixelPerfect: bool, }

impl App {
    pub fn new(title: ~str, width: uint, height: uint) -> Result<~App, ~str> {
        sdl2::init([sdl2::InitVideo]);

        let window = match video::Window::new(title,
            sdl2::video::PosCentered, sdl2::video::PosCentered,
            width as int, height as int, [video::Shown, video::Resizable]) {
            Ok(window) => window,
            Err(err) => return Err(format!("SDL2 window fail: {}", err))
        };
        let renderer = match render::Renderer::from_window(window,
            render::DriverAuto, [render::Accelerated, render::PresentVSync]) {
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
            width: width,
            height: height,
            pixels: pixels,
            pixelPerfect: true,
        })
    }

    fn targetRect(&self) -> Rect {
        let port = self.renderer.get_viewport();

        let (w, h) = targetSize(self.width, self.height, &port, self.pixelPerfect);
        Rect{
            x: ((port.w as uint - w) / 2) as i32, y: ((port.h as uint - h) / 2) as i32,
            w: w as i32, h: h as i32
        }
    }

    pub fn render(&self) {
        self.renderer.clear();
        self.texture.update(None, self.pixels, self.width as int * 4);
        self.renderer.copy(self.texture, None, Some(self.targetRect()));
        self.renderer.present();
    }
}

fn targetSize(width: uint, height: uint, port: &Rect, pixelPerfect: bool) -> (uint, uint) {
    let wScale = port.w as f64 / width as f64;
    let hScale = port.h as f64 / height as f64;
    let minScale = if wScale < hScale { wScale } else { hScale };

    if minScale < 1.0 || !pixelPerfect {
         // Less than 1 physical pixel for 1 logical pixel, can't achieve
         // pixel perfection so just scale to what we get.
        ((width as f64 * minScale) as uint,
         (height as f64 * minScale) as uint)
    } else {
        ((width as f64 * minScale.floor()) as uint,
         (height as f64 * minScale.floor()) as uint)
    }
}
