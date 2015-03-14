use std::fmt::Write;
use util::{timing, color, Anchor, V2};
use backend::{Canvas, Event, Key, Fonter};

pub struct Console {
    text: String,
    command: String,
}

impl Console {
    pub fn new() -> Console {
        Console {
            text: "Welcome to console\n".to_string(),
            command: String::new(),
        }
    }

    pub fn update(&mut self, ctx: &mut Canvas) {
        let mut full_text = self.text.clone() + "]" + &self.command[..];

        // Blinking cursor.
        full_text = full_text + if timing::spike(0.5, 0.5) { "_" } else { " " };

        Fonter::new(ctx)
            .color(&color::LIGHTGREEN).border(&color::BLACK)
            .anchor(Anchor::BottomLeft)
            .text(full_text)
            .draw(V2(0.0, 180.0));
    }

    pub fn process(&mut self, event: Event) -> bool {
        match event {
            // Close the console by typing grave again. Can't actually type it
            // in console commands then unless we add escape keys.
            Event::Char('`') => { return false; }
            Event::KeyPressed(Key::Escape) => { return false; }

            Event::KeyPressed(Key::Backspace) => {
                let len = self.command.len();
                if len > 0 {
                    // XXX: This is very bad juju if the string isn't entirely
                    // ASCII7.
                    self.command.truncate(len - 1);
                }
            }
            Event::KeyPressed(Key::Enter) => { self.process_command(); }
            Event::KeyPressed(Key::Tab) => { self.tab_complete(); }
            Event::KeyPressed(Key::Up) => { self.history_prev(); }
            Event::KeyPressed(Key::Down) => { self.history_next(); }

            Event::Char(ch) if (ch as i32) >= 32 && (ch as i32) < 128 => {
                self.command.push(ch);
            }

            Event::Render(ctx) => { self.update(ctx); }
            _ => {}
        }

        return true;
    }

    fn process_command(&mut self) {
        // TODO: Why can't I use write! macros here?
        //writeln!(self.text, "]{}", self.command).unwrap();
        //writeln!(self.text, "TODO: Handle input");
        self.text.write_str(&format!("]{}\n", self.command)[..]).unwrap();
        self.text.write_str("TODO: Handle input\n").unwrap();
        self.command = "".to_string();
    }

    /// Try to tab-complete the current input string.
    fn tab_complete(&mut self) {
        // TODO
    }

    /// Bring the previous command in command history to current prompt.
    fn history_prev(&mut self) {
        // TODO
    }

    /// Bring the next command in command history to current prompt.
    fn history_next(&mut self) {
        // TODO
    }
}
