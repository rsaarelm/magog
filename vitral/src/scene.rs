use crate::atlas_cache::AtlasCache;
use crate::backend::Backend;
use crate::keycode::Keycode;
use crate::{Canvas, FontData, ImageBuffer, ImageData, SubImageSpec, UiState};
use crate::{Flick, FLICKS_PER_SECOND};
use euclid::default::Size2D;
use euclid::size2;
use std::error::Error;
use std::sync::Mutex;

pub type ImageKey = SubImageSpec<String>;

/// Toplevel type for current program GUI state.
///
/// Parametrized on a shared application state type T.
pub trait Scene<T> {
    /// Update the logic for this scene.
    ///
    /// May be called several times for one `render` call. Return value tells if this update should
    /// be followed by a change of scene.
    fn update(&mut self, _ctx: &mut T) -> Option<SceneSwitch<T>> { None }

    /// Draw this scene to a Vitral canvas.
    ///
    /// Render is separate from update for frame rate regulation reasons. If drawing is slow,
    /// multiple updates will be run for one render call.
    ///
    /// The render method can also introduce scene transitions in case there is immediate mode GUI
    /// logic written in the render code.
    fn render(&mut self, _ctx: &mut T, _canvas: &mut Canvas) -> Option<SceneSwitch<T>> { None }

    /// Process an input event.
    fn input(
        &mut self,
        _ctx: &mut T,
        _event: &InputEvent,
        _canvas: &mut Canvas,
    ) -> Option<SceneSwitch<T>> {
        None
    }

    /// Return true if the scene below this one in the scene stack should be visible.
    ///
    /// Is true for scenes that implement a pop-up element instead of a full-screen scene.
    fn draw_previous(&self) -> bool { false }
}

/// Scene transition description.
pub enum SceneSwitch<T> {
    /// Exit from current scene and return to the previous one on the scene stack.
    Pop,
    /// Push a new scene on top of this one.
    Push(Box<dyn Scene<T>>),
    /// Replace this one with a different scene on the top of the stack.
    Replace(Box<dyn Scene<T>>),
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    Typed(char),
    KeyEvent {
        is_down: bool,
        key: Option<Keycode>,
        hardware_key: Option<Keycode>,
    },
}

pub struct AppConfig {
    pub frame_duration: Flick,
    pub resolution: Size2D<u32>,
    pub window_title: String,
}

impl AppConfig {
    pub fn new(title: impl Into<String>) -> AppConfig {
        AppConfig {
            frame_duration: Flick(FLICKS_PER_SECOND / 30),
            resolution: size2(640, 360),
            window_title: title.into(),
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
}

lazy_static! {
    /// Global game engine state.
    static ref ENGINE_STATE: Mutex<EngineState> = { Mutex::new(EngineState::default()) };
}

/// Start running a Scene state machine app with the given configuration.
pub fn run_app<T>(
    config: AppConfig,
    world: T,
    scenes: Vec<Box<dyn Scene<T>>>,
) -> Result<(), Box<dyn Error>> {
    // TODO: Maybe want world to come in as &mut T instead of T in the future so that the caller
    // can access it after run_app finishes? Or just return the world if exit is successful?
    let backend = Backend::start(
        config.resolution.width,
        config.resolution.height,
        config.window_title,
    )?;

    let mut gameloop = GameLoop::new(backend, world, scenes).frame_duration(config.frame_duration);

    gameloop.run();

    Ok(())
}

/// Saves a screenshot with the given name prefix to disk.
///
/// Panics if called when an app isn't running via `run_app`.
pub fn save_screenshot(_prefix: &str) -> Result<(), Box<dyn Error>> {
    unimplemented!();
}

/// Return the average frame duration for recent frames.
///
/// Panics if called when an app isn't running via `run_app`.
pub fn get_frame_duration() -> Flick {
    unimplemented!();
}

/// Add a named image into the engine image atlas.
pub fn add_sheet(id: impl Into<String>, sheet: impl Into<ImageBuffer>) -> ImageKey {
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
    sheet: impl Into<ImageBuffer>,
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
    sheet: impl Into<ImageBuffer>,
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
    // TODO: Convert AtlasCache::get to match this API instead of panicing at failure
    // (Best done when conversion to new gameloop is done and AtlasCache API isn't used publicly
    // anymore.)
    Some(ENGINE_STATE.lock().unwrap().atlas_cache.get(key).clone())
}

/// `Scene` stack based game loop runner.
pub struct GameLoop<T> {
    frame_duration: Flick,
    scene_stack: Vec<Box<dyn Scene<T>>>,
    world: T,
    backend: Backend,
    ui: UiState,
}

impl<T> GameLoop<T> {
    pub fn new(backend: Backend, world: T, scenes: Vec<Box<dyn Scene<T>>>) -> GameLoop<T> {
        GameLoop {
            frame_duration: Flick(FLICKS_PER_SECOND / 30),
            scene_stack: scenes,
            world,
            backend,
            ui: UiState::default(),
        }
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

    pub fn run(&mut self) {
        // Inspired by https://gafferongames.com/post/fix_your_timestep/
        let mut t = Flick::now();
        let mut accum = Flick(0);
        'gameloop: loop {
            let new_t = Flick::now();
            let frame_duration = new_t - t;
            t = new_t;

            accum += frame_duration;

            while accum >= self.frame_duration {
                self.update();
                accum -= self.frame_duration;
                if self.scene_stack.is_empty() {
                    break 'gameloop;
                }
            }

            self.backend
                .sync_with_atlas_cache(&mut ENGINE_STATE.lock().unwrap().atlas_cache);

            self.render();

            if self.scene_stack.is_empty() {
                break 'gameloop;
            }

            if let Ok(events) = self.backend.process_events(&mut self.ui) {
                for event in &events {
                    let idx = self.scene_stack.len() - 1;
                    let ret = {
                        let mut canvas = Canvas::new(
                            self.backend.canvas_size(),
                            &mut self.ui,
                            &mut self.backend,
                        );
                        self.scene_stack[idx].input(&mut self.world, event, &mut canvas)
                    };
                    self.process(ret);

                    if self.scene_stack.is_empty() {
                        break 'gameloop;
                    }
                }
            } else {
                // Window closed
                break 'gameloop;
            }
        }
    }
}
