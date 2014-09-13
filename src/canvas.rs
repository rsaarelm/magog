use time;
use std::mem;
use sync::comm::Receiver;
use image::{GenericImage, Pixel, ImageBuf, Rgba};
use glfw;
use glfw::Context as _Context;
use gfx;

pub struct Canvas {
    title: String,
    dim: [u32, ..2],
    frame_interval: Option<f64>,

    image_collector: ImageCollector,
}

/// Toplevel graphics drawing and input reading context.
impl Canvas {
    pub fn new() -> Canvas {
        Canvas {
            title: "window".to_string(),
            dim: [640, 360],
            frame_interval: None,
            image_collector: ImageCollector::new(),
        }
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
                |(_x, _y, p)| p.to_rgba())
            .collect(),
            w, h);
        self.pending_images.push(img);

        self.next_idx += 1;
        self.next_idx - 1
    }

    pub fn make_atlas(&mut self) {
        unimplemented!();
    }
}
