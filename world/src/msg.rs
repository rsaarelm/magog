use std::cell::RefCell;

local_data_key!(MSG_QUEUE: RefCell<Vec<::Msg>>)

/// Pop and return the oldest message left in the message queue.
pub fn pop_msg() -> Option<::Msg> {
    if MSG_QUEUE.get().is_none() { return None; }
    MSG_QUEUE.get().unwrap().borrow_mut().remove(0)
}

/// Insert a new message to the back of the message queue.
pub fn push_msg(msg: ::Msg) {
    if MSG_QUEUE.get().is_none() {
        MSG_QUEUE.replace(Some(RefCell::new(vec![msg])));
    } else {
        MSG_QUEUE.get().unwrap().borrow_mut().push(msg);
    }
}

// TODO: A println! style formatting msg! macro that emits text messages and
// optionally supports special game system stuff like entity name
// interpolation and string coloring.
