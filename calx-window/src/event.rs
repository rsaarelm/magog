use super::Key;

/// Canvas event.
#[derive(Copy, Clone, PartialEq)]
pub enum Event {
    RenderFrame,
    Quit,
    Char(char),
    KeyPress(Key),
    KeyRelease(Key),
    MouseMove([f32; 2]),
    MouseWheel(i32),
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    /// A click is a rapid press and release of the mouse.
    MouseClick(MouseButton),
    /// Ongoing mouse drag event.
    ///
    /// A drag is a movement of the mouse while a button is pressed.
    MouseDrag(MouseButton, [f32; 2], [f32; 2]),
    /// A drag that ended with the button being released.
    MouseDragEnd(MouseButton, [f32; 2], [f32; 2]),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}
