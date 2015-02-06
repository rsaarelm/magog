use canvas::Canvas;

pub enum Event<'a> {
    /// Time to render the screen. Call your own render code on the Canvas
    /// value when you get this.
    Render(&'a mut Canvas),
    Char(char),
    KeyPressed(::Key),
    KeyReleased(::Key),
    MouseMoved((i32, i32)),
    MouseWheel(i32),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    /// The window has changed focus. True if gained, false if lost.
    FocusChanged(bool),
}

#[derive(Copy, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}
