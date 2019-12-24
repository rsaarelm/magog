use crate::grammar;

/// Message receiver that is implemented in client
pub trait MsgReceiver: Sync + Send {
    fn msg(&self, text: &str);
}

pub(crate) static mut MSG_RECEIVER: &dyn MsgReceiver = &StdoutReceiver;

/// Register the message receiver used by the game frontend.
pub fn register_msg_receiver(receiver: Box<dyn MsgReceiver>) {
    unsafe {
        MSG_RECEIVER = Box::leak(receiver);
    }
}

pub(crate) fn grammatize(fmt: &str, elements: &[grammar::GrammarPart]) -> String {
    // Because of the macro system, element set is a loose bag of stuff. Expecting to find Subject
    // and Object in there if they are used in the message.
    use grammar::GrammarPart::*;
    let mut templater: Box<dyn grammar::Templater> = match elements {
        [] => Box::new(grammar::EmptyTemplater),
        [Subject(subject)] => Box::new(grammar::SubjectTemplater::new(subject.clone())),
        [Subject(subject), Object(object)] | [Object(object), Subject(subject)] => {
            Box::new(grammar::ObjectTemplater::new(
                grammar::SubjectTemplater::new(subject.clone()),
                object.clone(),
            ))
        }
        _ => {
            panic!("grammar block other than [], [subject], [subject, object]");
        }
    };

    templater.format(fmt).unwrap_or_else(|e| panic!("{}", e))
}

pub(crate) fn dispatch_msg(msg: &str) {
    unsafe {
        MSG_RECEIVER.msg(msg);
    }
}

#[macro_export]
macro_rules! msg {
    ($fmt:expr) => {
        $crate::msg::dispatch_msg($fmt);
    };

    ($fmt:expr, $($arg:expr),*) => {
        let __txt = format!($fmt, $($arg),*);
        $crate::msg::dispatch_msg(&__txt);
    };

    ($fmt:expr; $($grammar_arg:expr),*) => {
        let __txt = $crate::msg::grammatize($fmt, &[$($grammar_arg),*]);
        $crate::msg::dispatch_msg(&__txt);
    };

    ($fmt:expr, $($arg:expr),*; $($grammar_arg:expr),*) => {
        let __txt = format!($fmt, $($arg),*);
        let __txt = $crate::msg::grammatize(&__txt, &[$($grammar_arg),*]);
        $crate::msg::dispatch_msg(&__txt);
    };
}

struct StdoutReceiver;

impl MsgReceiver for StdoutReceiver {
    fn msg(&self, text: &str) {
        println!("{}", text);
    }
}
