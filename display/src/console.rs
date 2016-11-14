use std::cmp::min;
use std::io::prelude::*;
use std::io;
use std::str;
use std::mem;
use time;
use euclid::{Point2D, Rect};
use calx_resource::Resource;
use backend;


struct Message {
    expire_time_s: f64,
    text: String,
}

impl Message {
    fn new(text: String, time_start_s: f64) -> Message {
        const TIME_TO_READ_CHAR_S: f64 = 0.1;
        let expire_time_s = time_start_s + text.len() as f64 * TIME_TO_READ_CHAR_S;
        Message {
            expire_time_s: expire_time_s,
            text: text,
        }
    }
}

/// Output 
pub struct Console {
    lines: Vec<Message>,
    input_buffer: String,
    finished_input: Vec<u8>,
    output_buffer: String,
    done_reading_s: f64,
}

impl Console {
    pub fn new() -> Console {
        Console {
            lines: Vec::new(),
            input_buffer: String::new(),
            finished_input: Vec::new(),
            output_buffer: String::new(),
            done_reading_s: 0.0,
        }
    }

    /// Draw the console as a regular message display.
    pub fn draw_small(&mut self, context: &mut backend::Context, screen_area: &Rect<f32>) {
        // TODO: Store default font in context object
        let font: Resource<backend::Font> = Resource::new("default".to_string()).unwrap();
        // TODO: Ditto for color
        let color = [1.0, 1.0, 1.0, 1.0];

        let t = time::precise_time_s();
        let mut y = font.0.height;
        for visible in self.lines.iter().filter(|m| m.expire_time_s > t) {
            // TODO: Split messages longer than screen_area width.
            context.ui.draw_text(&font.0, Point2D::new(0.0, y), color, &visible.text);
            y += font.0.height;
        }
    }

    /// Draw the console as a big drop-down with a command prompt.
    pub fn draw_large(&mut self, context: &mut backend::Context, screen_area: &Rect<f32>) {
        // TODO: Store default font in context object
        let font: Resource<backend::Font> = Resource::new("default".to_string()).unwrap();
        // TODO: Ditto for color
        let color = [1.0, 1.0, 1.0, 1.0];

        // TODO: Print the last lines of output.
        context.ui.draw_text(&font.0,
                             Point2D::new(0.0, font.0.height),
                             color,
                             "Hello world! (TODO: This should be a big wide thing)");
        // TODO: Bring up the command prompt here.
    }

    fn end_message(&mut self) {
        let mut message_text = String::new();
        mem::swap(&mut message_text, &mut self.output_buffer);

        let now = time::precise_time_s();
        if now > self.done_reading_s {
            self.done_reading_s = now;
        }

        let message = Message::new(message_text, self.done_reading_s);
        assert!(message.expire_time_s >= self.done_reading_s);
        self.done_reading_s = message.expire_time_s;
        self.lines.push(message);
    }
}

impl Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = min(buf.len(), self.finished_input.len());
        self.finished_input.drain(..len).enumerate().map(|(i, b)| buf[i] = b);
        Ok(len)
    }
}

impl Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = str::from_utf8(buf).expect("Wrote non-UTF-8 to Console");
        let mut lines = s.split('\n');
        lines.next().map(|text| self.output_buffer.push_str(text));

        for line in lines {
            self.end_message();
            self.output_buffer.push_str(line);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
