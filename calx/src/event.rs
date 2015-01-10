use canvas::Context;

pub enum Event<'a> {
    /// Time to render the screen. Call your own render code on the Context
    /// value when you get this.
    Render(&'a mut Context),
    Char(char),
    KeyPressed(::Key),
    KeyReleased(::Key),
}

