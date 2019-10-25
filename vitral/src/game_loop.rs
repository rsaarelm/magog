use crate::atlas_cache::AtlasCache;
use crate::backend::Backend;
use crate::keycode::Keycode;
use crate::scene::Scene;
use crate::scene::SceneSwitch;
use crate::{Canvas, FontData, ImageData, InputEvent, MouseButton, SubImageSpec, UiState};
use crate::{Flick, FLICKS_PER_SECOND};
use euclid::default::Size2D;
use euclid::{point2, size2};
use image::RgbaImage;
use lazy_static::lazy_static;
use log::info;
use log::{debug, warn};
use std::sync::Mutex;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub type ImageKey = SubImageSpec<String>;

pub struct AppConfig {
    pub frame_duration: Flick,
    pub resolution: Size2D<u32>,
    pub window_title: String,
    /// If true, scale graphics only in integer multiples of pixel size.
    ///
    /// All pixels on a pixel-perfect window will have equal size, which prevents scaling artifacts
    /// from messing up pixel art, but there may also be large unused areas on the borders of the
    /// application window.
    pub pixel_perfect: bool,
}

impl AppConfig {
    pub fn new(title: impl Into<String>) -> AppConfig {
        AppConfig {
            frame_duration: Flick(FLICKS_PER_SECOND / 30),
            resolution: size2(640, 360),
            window_title: title.into(),
            pixel_perfect: true,
        }
    }

    pub fn frame_duration(mut self, frame_duration: Flick) -> AppConfig {
        self.frame_duration = frame_duration;
        self
    }
}

#[derive(Default)]
struct EngineState {
    atlas_cache: AtlasCache<String>,
    average_frame_duration: Flick,
}

lazy_static! {
    /// Global game engine state.
    static ref ENGINE_STATE: Mutex<EngineState> = { Mutex::new(EngineState::default()) };
}

/// Grow the window so it'll fit the current monitor.
fn window_geometry<T>(
    resolution: Size2D<u32>,
    event_loop: &EventLoop<T>,
) -> (LogicalPosition, LogicalSize) {
    // Don't make it a completely fullscreen window, that might put the window title bar
    // outside the screen.
    const BUFFER: f64 = 8.0;
    let width = resolution.width as f64;
    let height = resolution.height as f64;

    let monitor_size = event_loop.primary_monitor().size();
    // Get the most conservative DPI if there's a weird multi-monitor setup.
    let dpi_factor = event_loop
        .available_monitors()
        .map(|m| m.hidpi_factor())
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .expect("No monitors found!");
    info!("Scaling starting size to monitor");
    info!("Monitor size {:?}", monitor_size);
    info!("DPI Factor {}", dpi_factor);

    let mut window_size = PhysicalSize::new(width, height);
    while window_size.width + width <= monitor_size.width - BUFFER
        && window_size.height + height <= monitor_size.height - BUFFER
    {
        window_size.width += width;
        window_size.height += height;
    }
    info!("Adjusted window size: {:?}", window_size);
    let window_pos = PhysicalPosition::new(
        (monitor_size.width - window_size.width) / 2.0,
        (monitor_size.height - window_size.height) / 2.0,
    );

    (
        window_pos.to_logical(dpi_factor),
        window_size.to_logical(dpi_factor),
    )
}

/// Start running a Scene state machine app with the given configuration.
pub fn run_app<T: 'static>(config: AppConfig, world: T, scenes: Vec<Box<dyn Scene<T>>>) -> ! {
    // TODO: Maybe want world to come in as &mut T instead of T in the future so that the caller
    // can access it after run_app finishes? Or just return the world if exit is successful?

    let event_loop = EventLoop::new();
    let (_pos, size) = window_geometry(config.resolution, &event_loop);
    // TODO: Use Window directly here when using wgpu?
    //let window = WindowBuilder::new()
    //    .with_title(config.window_title)
    //    .with_inner_size(size)
    //    .build(&event_loop)
    //    .unwrap();
    //window.set_outer_position(pos);

    let backend: Backend = Backend::start(
        &event_loop,
        WindowBuilder::new()
            .with_title(config.window_title)
            .with_inner_size(size),
        config.resolution.width,
        config.resolution.height,
        config.pixel_perfect,
    )
    .unwrap();

    GameLoop::new(backend, world, scenes)
        .frame_duration(config.frame_duration)
        .run(event_loop)
}

/// Return the average frame duration for recent frames.
///
/// Panics if called when an app isn't running via `run_app`.
pub fn get_frame_duration() -> Flick { ENGINE_STATE.lock().unwrap().average_frame_duration }

/// Add a named image into the engine image atlas.
pub fn add_sheet(id: impl Into<String>, sheet: impl Into<RgbaImage>) -> ImageKey {
    ENGINE_STATE
        .lock()
        .unwrap()
        .atlas_cache
        .add_sheet(id, sheet)
}

/// Add a tilesheet image that gets automatically split to subimages based on image structure.
///
/// Tiles are bounding boxes of non-background pixel groups surrounded by only background pixels or
/// image edges. Background color is the color of the bottom right corner pixel of the image. The
/// bounding boxes are returned lexically sorted by the coordinates of their bottom right corners,
/// first along the y-axis then along the x-axis. This produces a natural left-to-right,
/// bottom-to-top ordering for a cleanly laid out tile sheet.
///
/// Note that the background color is a solid color, not transparent pixels. The inner tiles may
/// have transparent parts, so a solid color is needed to separate them.
pub fn add_tilesheet(
    id: impl Into<String>,
    sheet: impl Into<RgbaImage>,
    _span: impl IntoIterator<Item = char>,
) -> Vec<ImageKey> {
    ENGINE_STATE
        .lock()
        .unwrap()
        .atlas_cache
        .add_tilesheet(id, sheet)
}

/// Add a bitmap font read from a tilesheet image.
pub fn add_tilesheet_font(
    id: impl Into<String>,
    sheet: impl Into<RgbaImage>,
    span: impl IntoIterator<Item = char>,
) -> FontData {
    ENGINE_STATE
        .lock()
        .unwrap()
        .atlas_cache
        .add_tilesheet_font(id, sheet, span)
}

/// Get a drawable (sub)image from the cache corresponding to the given `ImageKey`.
///
/// If the `ImageKey` specifies a sheet not found in the cache or invalid dimensions, will return
/// `None`.
pub fn get_image(key: &ImageKey) -> Option<ImageData> {
    ENGINE_STATE.lock().unwrap().atlas_cache.get(key)
}

/// `Scene` stack based game loop runner.
pub struct GameLoop<T> {
    frame_duration: Flick,
    scene_stack: Vec<Box<dyn Scene<T>>>,
    world: T,
    backend: Backend,
    ui: UiState,

    last_frame_time: Flick,
    accumulated_update_time: Flick,
    average_frame_duration: Flick,
    exit_requested: bool,
}

impl<T: 'static> GameLoop<T> {
    pub fn new(backend: Backend, world: T, scenes: Vec<Box<dyn Scene<T>>>) -> GameLoop<T> {
        let frame_duration = Flick(FLICKS_PER_SECOND / 30);
        GameLoop {
            frame_duration,
            scene_stack: scenes,
            world,
            backend,
            ui: UiState::default(),

            last_frame_time: Flick::now(),
            accumulated_update_time: Flick::default(),
            average_frame_duration: frame_duration,
            exit_requested: false,
        }
    }

    pub fn run<U>(mut self, event_loop: EventLoop<U>) -> ! {
        let mut input_events = Vec::new();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let position = position
                            .to_physical(self.backend.display.gl_window().window().hidpi_factor());
                        let pos = self.backend.zoom.screen_to_canvas(
                            self.backend.window_size,
                            self.backend.render_buffer.size(),
                            point2(position.x as f32, position.y as f32),
                        );
                        self.ui.input_mouse_move(pos.x as i32, pos.y as i32);
                    }
                    WindowEvent::MouseInput { state, button, .. } => self.ui.input_mouse_button(
                        match button {
                            winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            _ => MouseButton::Middle,
                        },
                        state == ElementState::Pressed,
                    ),
                    WindowEvent::ReceivedCharacter(c) => {
                        input_events.push(InputEvent::Typed(c));
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                scancode,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => {
                        let is_down = state == ElementState::Pressed;
                        let key = virtual_keycode
                            .and_then(|virtual_keycode| Keycode::try_from(virtual_keycode).ok());
                        // Winit adjusts the Linux scancodes, take into account. Don't know if
                        // this belongs here in the glium module or in the Keycode translation
                        // maps...
                        let scancode = if cfg!(target_os = "linux") {
                            scancode + 8
                        } else {
                            scancode
                        };
                        let hardware_key = Keycode::from_scancode(scancode);
                        if key.is_some() || hardware_key.is_some() {
                            input_events.push(InputEvent::KeyEvent {
                                is_down,
                                key,
                                hardware_key,
                            });
                        }
                    }
                    _ => (),
                },
                Event::DeviceEvent { .. } => {}
                Event::UserEvent(_) => {}
                Event::NewEvents(_) => {}
                Event::LoopDestroyed => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::Suspended => {
                    // TODO: Suspend loop.
                }
                Event::Resumed => {}
                Event::EventsCleared => {
                    self.update_step();
                    self.render_step();

                    for event in input_events.drain(0..) {
                        self.process_event(event);
                    }

                    if self.exit_requested() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
        })
    }

    pub fn frame_duration(mut self, frame_duration: Flick) -> Self {
        self.frame_duration = frame_duration;
        self
    }

    fn update(&mut self) {
        let top = self.scene_stack.len() - 1;
        let ret = self.scene_stack[top].update(&mut self.world);
        self.process(ret);
    }

    fn process(&mut self, switch: Option<SceneSwitch<T>>) {
        let top = self.scene_stack.len() - 1;
        match switch {
            Some(SceneSwitch::Pop) => {
                self.scene_stack.pop();
            }
            Some(SceneSwitch::Push(scene)) => {
                self.scene_stack.push(scene);
            }
            Some(SceneSwitch::Replace(scene)) => {
                self.scene_stack[top] = scene;
            }
            None => {}
        }
    }

    fn render(&mut self) {
        // Find the lowest scene from top with only transparent scenes on top of it.
        let end = self.scene_stack.len();
        let mut begin = end - 1;
        while begin > 0 && self.scene_stack[begin].draw_previous() {
            begin -= 1;
        }

        let mut switch = None;
        let draw_list = {
            let mut canvas =
                Canvas::new(self.backend.canvas_size(), &mut self.ui, &mut self.backend);

            for i in begin..end {
                // Only the switch result from the topmost state counts here.
                switch = self.scene_stack[i].render(&mut self.world, &mut canvas);
            }
            canvas.end_frame()
        };
        self.process(switch);
        self.backend.render(&draw_list);
    }

    fn update_step(&mut self) {
        // Inspired by https://gafferongames.com/post/fix_your_timestep/

        let new_t = Flick::now();
        let frame_duration = new_t - self.last_frame_time;

        self.average_frame_duration = Flick(
            (0.95 * self.average_frame_duration.0 as f64 + 0.05 * frame_duration.0 as f64) as i64,
        );
        ENGINE_STATE.lock().unwrap().average_frame_duration = self.average_frame_duration;
        debug!(
            "FPS {:.1}",
            FLICKS_PER_SECOND as f64 / self.average_frame_duration.0 as f64
        );

        self.last_frame_time = new_t;
        self.accumulated_update_time += frame_duration;

        while self.accumulated_update_time >= self.frame_duration {
            let update_time = Flick::now();
            self.update();
            let update_time = Flick::now() - update_time;
            let update_ratio = update_time.0 as f64 / self.frame_duration.0 as f64;

            // If a single update takes most of the frame time, things are bad. Game loop may
            // enter a death spiral where every cycle needs more updates and makes the lag
            // worse. If an over-long update is detected, cut things short and flush the
            // accumulator.
            //
            // XXX: This has not been tested.
            if update_ratio > 0.9 {
                warn!(
                    "Scene update took {} % of alotted frame time. Skipping further updates",
                    (update_ratio * 100.0) as i32
                );

                self.accumulated_update_time = Flick(0);
                break;
            }

            self.accumulated_update_time -= self.frame_duration;
            if self.scene_stack.is_empty() {
                self.exit_requested = true;
            }
        }
    }

    fn render_step(&mut self) {
        self.backend
            .sync_with_atlas_cache(&mut ENGINE_STATE.lock().unwrap().atlas_cache);

        let render_time = Flick::now();
        self.render();
        let render_time = Flick::now() - render_time;
        let render_ratio = render_time.0 as f64 / self.frame_duration.0 as f64;

        // Renders can be expected to go over budget occasionally, but it's still a problem if
        // you want to keep a steady FPS, so make some log noise.
        if render_ratio > 1.0 {
            debug!(
                "Scene render too {} % of alotted frame time",
                (render_ratio * 100.0) as i32
            );
        }

        if self.scene_stack.is_empty() {
            self.exit_requested = true;
        }
    }

    pub fn process_event(&mut self, event: InputEvent) {
        let idx = self.scene_stack.len() - 1;
        let ret = {
            let mut canvas =
                Canvas::new(self.backend.canvas_size(), &mut self.ui, &mut self.backend);
            self.scene_stack[idx].input(&mut self.world, &event, &mut canvas)
        };
        self.process(ret);

        if self.scene_stack.is_empty() {
            self.exit_requested = true;
        }
    }

    /*
    pub fn run(mut self) {
        self.last_frame_time = Flick::now();
        self.accumulated_update_time = Flick(0);

        self.backend.run(move |input_events| {
            self.update_step();
            self.render_step();

            for event in &input_events {
                let idx = self.scene_stack.len() - 1;
                let ret = {
                    let mut canvas =
                        Canvas::new(self.backend.canvas_size(), &mut self.ui, &mut self.backend);
                    self.scene_stack[idx].input(&mut self.world, event, &mut canvas)
                };
                self.process(ret);

                if self.scene_stack.is_empty() {
                    self.exit_requested = true;
                }
            }

            return !self.exit_requested;
        });
    }
    */

    pub fn exit_requested(&self) -> bool { self.exit_requested }
}
