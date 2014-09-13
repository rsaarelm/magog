use std::collections::hashmap::HashMap;
use time;
use std::mem;
use sync::comm::Receiver;
use image::{GenericImage, SubImage, Pixel, ImageBuf, Rgba};
use image;
use glfw;
use glfw::Context as _Context;
use gfx;

static FONT_DATA: &'static [u8] = include_bin!("../assets/font.png");

/// The magic RGB value that means a pixel should be set to transparent. The
/// current (as of 2014-09-13) image library doesn't seem to see palettized PNG
/// images with an alpha key set as RGBA, so I need to resort to trickery to get
/// myself a nice RGBA source from the image. Of course this is totally
/// unportable for any purpose where images might want to use the colorkey color
/// for something that's not a transparency layer.
pub static COLOR_KEY_RGB: (u8, u8, u8) = (0x80, 0x80, 0x80);

pub struct Canvas {
    title: String,
    dim: [u32, ..2],
    frame_interval: Option<f64>,
    image_collector: ImageCollector,
    font_glyphs: HashMap<char, Image>,
}

/// Toplevel graphics drawing and input reading context.
impl Canvas {
    pub fn new() -> Canvas {
        let mut ret = Canvas {
            title: "window".to_string(),
            dim: [640, 360],
            frame_interval: None,
            image_collector: ImageCollector::new(),
            font_glyphs: HashMap::new(),
        };
        ret.init_font();
        ret
    }

    pub fn set_title(&mut self, title: &str) -> &mut Canvas {
        self.title = title.to_string();
        self
    }

    /// Set the frame rate.
    pub fn set_frame_interval(&mut self, interval_s: f64) -> &mut Canvas {
        assert!(interval_s > 0.00001);
        self.frame_interval = Some(interval_s);
        self
    }

    /// Set the resolution.
    pub fn set_dim(&mut self, dim: [u32, ..2]) -> &mut Canvas {
        self.dim = dim;
        self
    }

    pub fn add_image<P: Pixel<u8>, I: GenericImage<P>>(
        &mut self, image: I) -> Image {
        Image(self.image_collector.push(image))
    }

    /// Start running the engine, return an event iteration.
    pub fn run(&mut self) -> Context {
        // TODO: Make atlas with image_collector, pass it to context.
        Context::new(
            self.dim,
            self.title.as_slice(),
            Some(1.0 / 30.0))
    }

    fn init_font(&mut self) {
        let mut font_sheet = image::load_from_memory(FONT_DATA, image::PNG).unwrap();
        for i in range(0u32, 96u32) {
            let x = 8u32 * (i % 16u32);
            let y = 8u32 * (i / 16u32);
            let glyph = self.add_image(SubImage::new(&mut font_sheet, x, y, 8, 8));
            self.font_glyphs.insert((i + 32) as u8 as char, glyph);
        }
    }
}

pub struct Context {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    graphics: gfx::Graphics<gfx::GlDevice, gfx::GlCommandBuffer>,
    frame: gfx::Frame,

    state: State,
    frame_interval: Option<f64>,
    last_render_time: f64,
}

#[deriving(PartialEq)]
enum State {
    Normal,
    EndFrame,
}

impl Context {
    fn new(
        dim: [u32, ..2],
        title: &str,
        frame_interval: Option<f64>) -> Context {

        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (window, events) = glfw
            .create_window(dim[0], dim[1],
            title.as_slice(), glfw::Windowed)
            .expect("Failed to open window");
        window.make_current();
        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
        window.set_key_polling(true);
        window.set_char_polling(true);

        let device = gfx::GlDevice::new(|s| window.get_proc_address(s));
        let graphics = gfx::Graphics::new(device);
        let frame = gfx::Frame::new(dim[0] as u16, dim[1] as u16);

        Context {
            glfw: glfw,
            window: window,
            events: events,
            graphics: graphics,
            frame: frame,
            state: Normal,
            frame_interval: frame_interval,
            last_render_time: time::precise_time_s(),
        }
    }

    /// Clear the screen
    pub fn clear(&mut self, color: [f32, ..4]) {
        self.graphics.clear(
            gfx::ClearData {
                color: color,
                depth: 1.0,
                stencil: 0,
            }, gfx::Color, &self.frame);
    }
}

pub enum Event<'a> {
    Render(&'a mut Context),
    Input(int), // TODO: Proper input type
}

impl<'a> Iterator<Event<'a>> for Context {
    fn next(&mut self) -> Option<Event<'a>> {
        // After a render event, control will return here on a new
        // iter call. Do post-render work here.
        if self.state == EndFrame {
            self.state = Normal;
            self.graphics.end_frame();
            self.window.swap_buffers();
        }

        loop {
            if self.window.should_close() {
                return None;
            }

            self.glfw.poll_events();

            match self.events.try_recv() {
                Ok(_event) => {
                    // TODO: Process event.
                    return Some(Input(123))
                }
                _ => ()
            }

            let t = time::precise_time_s();
            if self.frame_interval.map_or(true,
                |x| t - self.last_render_time >= x) {
                self.last_render_time = t;

                // Time to render, must return a handle to self.
                // XXX: Need unsafe hackery to get around lifetimes check.
                self.state = EndFrame;
                unsafe {
                    return Some(Render(mem::transmute(self)))
                }
            }
        }
    }
}

/// Drawable images stored in the Canvas.
#[deriving(Clone, PartialEq)]
pub struct Image(uint);

struct ImageCollector {
    pending_images: Vec<ImageBuf<Rgba<u8>>>,
    next_idx: uint,
}

impl ImageCollector {
    fn new() -> ImageCollector {
        ImageCollector {
            pending_images: vec![],
            next_idx: 0,
        }
    }

    pub fn push<P: Pixel<u8>, I: GenericImage<P>>(
        &mut self, image: I) -> uint {
        let (w, h) = image.dimensions();
        let img = ImageBuf::from_pixels(
            image.pixels().map::<Rgba<u8>>(
                |(_x, _y, p)| self.convert_pixel(&p))
            .collect(),
            w, h);
        self.pending_images.push(img);

        self.next_idx += 1;
        self.next_idx - 1
    }

    pub fn convert_pixel<P: Pixel<u8>>(&self, pixel: &P) -> Rgba<u8> {
        let (r, g, b, mut a) = pixel.channels4();
        if (r, g, b) == COLOR_KEY_RGB && a == 0xff {
            a = 0;
        }

        Pixel::from_channels(r, g, b, a)
    }

    pub fn make_atlas(&mut self) {
        unimplemented!();
    }
}
