use time;
use backend::{Fonter, Context};
use util::{color, V2, text};

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
        let timeout = add_time_to_read(self.msg_done_time, text.as_slice());
        self.msgs.push(Msg::new(text, timeout));
        self.msg_done_time = Some(timeout);
    }

    pub fn caption(&mut self, text: String) {
        let timeout = add_time_to_read(self.caption_done_time, text.as_slice());
        // Showing up a caption for a thing after all the previous ones have
        // gone away doesn't look right. Just clearing the old captions for
        // now. A better approach might be showing several captions below each
        // other when multiple are live.
        self.captions.clear();
        self.captions.push(Msg::new(text, timeout));
        self.caption_done_time = Some(timeout);
    }

    fn draw_msgs(&self, ctx: &mut Context) {
        let msg_columns = 32;
        let msg_rows = 16;
        let msg_origin = V2(0, 360 - (msg_rows - 1) * 8);

        let mut writer = ctx.text_writer(msg_origin, 0.1, color::LIGHTGRAY)
            .set_border(color::BLACK);

        for msg in self.msgs.iter() {
            let _ = write!(&mut writer, "{}", text::wrap_lines(msg_columns, msg.text.as_slice()).as_slice());
        }
    }

    fn draw_caption(&self, ctx: &mut Context) {
        if !self.captions.is_empty() {
            let width = self.captions[0].text.len() as i32 * 8;
            let origin = V2(320 - width / 2, 180);
            // TODO: Wrap and center-justify multiline caption

            let mut writer = ctx.text_writer(origin, 0.1, color::LIGHTGRAY)
                .set_border(color::BLACK);

            let _ = write!(&mut writer, "{}", self.captions[0].text.as_slice());
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
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

