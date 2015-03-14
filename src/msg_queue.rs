use time;
use backend::{Fonter, Align, Canvas};
use util::{color, V2, Anchor};

struct Msg {
    pub text: String,
    /// Time point at which the Msg is phased out.
    pub timeout: f64,
}

impl Msg {
    fn new(text: String, timeout: f64) -> Msg {
        Msg {
            text: text,
            timeout: timeout,
        }
    }
}

pub struct MsgQueue {
    /// Regular messages that show up in the status panel
    msgs: Vec<Msg>,
    msg_done_time: Option<f64>,

    /// Special messages that show up one by one at the center of the screen.
    captions: Vec<Msg>,
    caption_done_time: Option<f64>,
}

impl MsgQueue {
    pub fn new() -> MsgQueue {
        MsgQueue {
            msgs: Vec::new(),
            msg_done_time: None,
            captions: Vec::new(),
            caption_done_time: None,
        }
    }

    pub fn msg(&mut self, text: String) {
        let timeout = add_time_to_read(self.msg_done_time, &text[..]);
        self.msgs.push(Msg::new(text, timeout));
        self.msg_done_time = Some(timeout);
    }

    pub fn caption(&mut self, text: String) {
        let timeout = add_time_to_read(self.caption_done_time, &text[..]);
        self.captions.push(Msg::new(text, timeout));
        self.caption_done_time = Some(timeout);
    }

    pub fn clear_captions(&mut self) {
        self.captions.clear();
    }

    fn draw_msgs(&self, ctx: &mut Canvas) {
        Fonter::new(ctx).color(&color::LIGHTGRAY).border(&color::BLACK)
            .width(320.0).max_lines(16)
            .anchor(Anchor::BottomLeft)
            .text(self.msgs.iter().fold(String::new(), |a, m| a + &m.text))
            .draw(V2(0.0, 360.0));
    }

    fn draw_caption(&self, ctx: &mut Canvas) {
        if !self.captions.is_empty() {
            Fonter::new(ctx).color(&color::LIGHTGRAY).border(&color::BLACK).width(160.0)
                .align(Align::Center).anchor(Anchor::Bottom)
                .text(self.captions[0].text.clone())
                .draw(V2(320.0, 172.0));
        }
    }

    pub fn draw(&self, ctx: &mut Canvas) {
        self.draw_msgs(ctx);
        self.draw_caption(ctx);
    }

    pub fn update(&mut self) {
        let t = time::precise_time_s();
        while !self.msgs.is_empty() {
            if self.msgs[0].timeout < t {
                self.msgs.remove(0);
            } else {
                break;
            }
        }
        if self.msgs.is_empty() { self.msg_done_time = None; }

        while !self.captions.is_empty() {
            if self.captions[0].timeout < t {
                self.captions.remove(0);
            } else {
                break;
            }
        }
        if self.captions.is_empty() { self.caption_done_time = None; }
    }
}

fn add_time_to_read(old_time: Option<f64>, text: &str) -> f64 {
    // Estimated time it takes the user to read one character in seconds.
    let letter_read_duration = 0.2;
    let read_time = text.trim().len() as f64 * letter_read_duration;

    return match old_time {
        Some(t) => t,
        None => time::precise_time_s()
    } + read_time;
}

