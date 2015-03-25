use std::cell::RefCell;
use super::{Msg};

thread_local!(static MSG_QUEUE: RefCell<Vec<Msg>> = RefCell::new(vec![]));

/// Pop and return the oldest message left in the message queue.
pub fn pop_msg() -> Option<Msg> {
    MSG_QUEUE.with(|q| {
        let empty = q.borrow().is_empty();
        if empty { None } else { Some(q.borrow_mut().remove(0)) }
    })
}

/// Insert a new message to the back of the message queue.
pub fn push(msg: Msg) {
    // XXX: Haven't figured out how to move values into Key::with blocks, so
    // need to use clone here.
    MSG_QUEUE.with(|q| q.borrow_mut().push(msg.clone()));
}
