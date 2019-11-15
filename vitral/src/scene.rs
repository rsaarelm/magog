use crate::{flick::Flick, keycode::Keycode, Canvas};

pub(crate) struct SceneStack<T> {
    stack: Vec<Box<dyn Scene<T>>>,
    frame_duration: Flick,
    t: Flick,
}

impl<T> SceneStack<T> {
    pub fn new(
        frame_duration: Flick,
        stack: Vec<Box<dyn Scene<T>>>,
    ) -> SceneStack<T> {
        SceneStack {
            stack,
            frame_duration,
            t: Flick::now(),
        }
    }

    pub fn apply(&mut self, switch: Option<SceneSwitch<T>>) {
        let top = self.stack.len() - 1;
        match switch {
            Some(SceneSwitch::Pop) => {
                self.stack.pop();
            }
            Some(SceneSwitch::Push(scene)) => {
                self.stack.push(scene);
            }
            Some(SceneSwitch::Replace(scene)) => {
                self.stack[top] = scene;
            }
            None => (),
        }
    }

    /// True if scene stack is empty and the application should be closed.
    pub fn is_empty(&self) -> bool { self.stack.is_empty() }

    /// Update stack based on accumulated time and apply scene changes.
    pub fn update(&mut self, ctx: &mut T) {
        let now = Flick::now();
        while (now - self.t) >= self.frame_duration {
            self.t += self.frame_duration;
            if self.is_empty() {
                continue;
            }

            let top = self.stack.len() - 1;
            let switch = self.stack[top].update(ctx);
            self.apply(switch);
        }
    }

    /// Render the stack of states and apply scene changes from the topmost scene.
    pub fn render(&mut self, ctx: &mut T, canvas: &mut Canvas) {
        if self.is_empty() {
            return;
        }
        let end = self.stack.len();
        let mut begin = end - 1;
        while begin > 0 && self.stack[begin].draw_previous() {
            begin -= 1;
        }

        let mut switch = None;
        for i in begin..end {
            // Only the switch result from the topmost state counts here.
            switch = self.stack[i].render(ctx, canvas);
        }
        self.apply(switch);
    }

    /// Process input events at the topmost state and apply scene changes.
    pub fn input(
        &mut self,
        ctx: &mut T,
        event: &InputEvent,
        canvas: &mut Canvas,
    ) {
        if self.is_empty() {
            return;
        }

        let idx = self.stack.len() - 1;
        let switch = self.stack[idx].input(ctx, event, canvas);
        self.apply(switch);
    }

    pub fn update_clock(&mut self) { self.t = Flick::now(); }
}

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
    fn render(
        &mut self,
        _ctx: &mut T,
        _canvas: &mut Canvas,
    ) -> Option<SceneSwitch<T>> {
        None
    }

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
