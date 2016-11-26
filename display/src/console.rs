use std::io::prelude::*;
use std::io;
use std::str;
use std::mem;
use time;
use euclid::{Point2D, Rect};
use calx_resource::Resource;
use calx_alg::split_line;
use backend;

struct Message {
    expire_time_s: f64,
    text: String,
}

impl Message {
    fn new(text: String, time_start_s: f64) -> Message {
        const TIME_TO_READ_CHAR_S: f64 = 0.2;
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
    output_buffer: String,
    done_reading_s: f64,
}

impl Console {
    pub fn new() -> Console {
        Console {
            lines: Vec::new(),
            input_buffer: String::new(),
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
        let mut y = screen_area.max_y();
        // The log can be very long, and we're always most interested in the latest ones, so
        // do a backwards iteration with an early exist once we hit a sufficiently old item.
        for msg in self.lines.iter().rev().take_while(|m| m.expire_time_s > t) {
            // The split_line iterator can't be reversed, need to do a bit of caching here.
            let fragments = split_line(&msg.text,
                                       |c| font.0.char_width(c).unwrap_or(0.0),
                                       screen_area.size.width)
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>();
            for line in fragments.iter().rev() {
                context.ui.draw_text(&font.0, Point2D::new(0.0, y), color, line);
                y -= font.0.height;
            }
        }
    }

    /// Draw the console as a big drop-down with a command prompt.
    pub fn draw_large(&mut self, context: &mut backend::Context, screen_area: &Rect<f32>) {
        // TODO: Store default font in context object
        let font: Resource<backend::Font> = Resource::new("default".to_string()).unwrap();
        // TODO: Ditto for color
        let color = [0.6, 0.6, 0.6, 1.0];
        let background = [0.0, 0.0, 0.6, 0.8];

        context.ui.fill_rect(*screen_area, background);

        let mut lines_left = (screen_area.size.height / font.0.height).ceil() as i32;

        let mut y = screen_area.max_y();

        // TODO: Handle enter with text input.
        // TODO: Command history.
        context.ui.text_input(&font.0, Point2D::new(0.0, y), color, &mut self.input_buffer);
        y -= font.0.height;
        lines_left -= 1;

        for msg in self.lines.iter().rev() {
            // XXX: Duplicated from draw_small.
            let fragments = split_line(&msg.text,
                                       |c| font.0.char_width(c).unwrap_or(0.0),
                                       screen_area.size.width)
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>();
            for line in fragments.iter().rev() {
                context.ui.draw_text(&font.0, Point2D::new(0.0, y), color, line);
                y -= font.0.height;
                lines_left -= 1;
            }

            if lines_left <= 0 {
                break;
            }
        }
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

    pub fn get_input(&mut self) -> String {
        let mut ret = String::new();
        mem::swap(&mut ret, &mut self.input_buffer);
        ret
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
