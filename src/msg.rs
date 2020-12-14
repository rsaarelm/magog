use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::Mutex;

#[derive(Clone, Default)]
pub struct MsgQueue {
    msgs: Vec<String>,
}

lazy_static! {
    pub static ref MSG_QUEUE: Mutex<RefCell<MsgQueue>> = Default::default();
}

struct QueueReceiver;

impl world::MsgReceiver for QueueReceiver {
    fn msg(&self, text: &str) {
        MSG_QUEUE
            .lock()
            .unwrap()
            .borrow_mut()
            .msgs
            .push(text.to_string());
    }
}

pub fn get() -> Vec<String> { std::mem::take(&mut MSG_QUEUE.lock().unwrap().borrow_mut().msgs) }

pub fn register() { world::register_msg_receiver(Box::new(QueueReceiver)); }
