use super::Key;

/// Canvas event.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Event {
    RenderFrame,
    Quit,
    Char(char),
    KeyPressed(Key),
    KeyReleased(Key),
    MouseMoved((i32, i32)),
    MouseWheel(i32),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    /// The window has changed focus. True if gained, false if lost.
    FocusChanged(bool),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}
