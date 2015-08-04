use std::cell::RefCell;
use std::collections::HashMap;
use time;

struct TimeLog {
    logs: HashMap<String, (u64, f64)>,
}

impl TimeLog {
    pub fn new() -> TimeLog {
        TimeLog {
            logs: HashMap::new(),
        }
    }

    pub fn log(name: String, mut duration: f64) {
        // TODO: Enable this when it's stable. Otherwise occasionally getting
        // 'access a TLS value during or after it is destroyed' errors when
        // exiting program and dumping the timing data.
        //if TIME_LOG.state() == LocalKeyState::Destroyed { return; }
        TIME_LOG.with(|a| {
            let mut n = 1;
            let contains = a.borrow().logs.contains_key(&name);
            if contains {
                let (old_n, total) = *a.borrow().logs.get(&name).unwrap();
                n = old_n + 1;
                duration += total;
            }

            a.borrow_mut().logs.insert(name, (n, duration));
        });
    }
}

impl Drop for TimeLog {
    fn drop(&mut self) {
        println!("Timing logs:");
        for (name, &(n, total)) in self.logs.iter() {
            println!("  {}:\t{:.3} s\t(avg. {:.3} s)", name, total, total / (n as f64));
        }
    }
}

thread_local!(static TIME_LOG: RefCell<TimeLog> = RefCell::new(TimeLog::new()));

#[must_use]
pub struct TimeLogItem {
    name: String,
    begin: f64,
}

impl TimeLogItem {
    pub fn new(name: &str) -> TimeLogItem {
        TimeLogItem {
            name: name.to_string(),
            begin: time::precise_time_s(),
        }
    }
}

impl Drop for TimeLogItem {
    fn drop(&mut self) {
        TimeLog::log(self.name.clone(), time::precise_time_s() - self.begin);
    }
}
