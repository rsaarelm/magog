use canvas::Context;
use key;

pub enum Event<'a> {
    /// Time to render the screen. Call your own render code on the Context
    /// value when you get this.
    Render(&'a mut Context),
    /// Some printable text entered. Data is whole strings to account for
    /// exotic input devices.
    Text(String),
    KeyPressed(key::Key),
    KeyReleased(key::Key),
}

