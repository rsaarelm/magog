use sync::comm::Receiver;
use glfw;
use glfw::Context as _Context;
use gfx;

pub struct Window {
    title: String,
    dim: [u32, ..2],
}

impl Window {
    pub fn new() -> Window {
        Window {
            title: "window".to_string(),
            dim: [640, 360],
        }
    }

    /// Start running the engine, return an event iteration.
    pub fn run(&self) -> Context {
        Context::new(self.dim, self.title.as_slice())
    }
}

pub struct Context {
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    device: gfx::GlDevice,
}

impl Context {
    fn new(dim: [u32, ..2], title: &str) -> Context {

        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (window, events) = glfw
            .create_window(dim[0], dim[1],
            title.as_slice(), glfw::Windowed)
            .expect("Failed to open window");
        window.make_current();
        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
        window.set_key_polling(true);
        window.set_char_polling(true);

        let device = gfx::GlDevice::new(|s| glfw.get_proc_address(s));

        Context {
            window: window,
            events: events,
            device: device,
        }
    }
}

pub enum Event {
    Render,
    Update,
    Input(int), // TODO: Proper input type
}

impl Iterator<Event> for Context {
    fn next(&mut self) -> Option<Event> {
        None
    }
}
