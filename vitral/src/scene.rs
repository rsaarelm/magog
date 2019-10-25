use crate::keycode::Keycode;
use crate::Canvas;

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
