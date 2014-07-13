use time;

pub struct ScopeTime {
    t: u64,
    desc: String,
}

local_data_key!(INDENT: uint)

fn indent() {
    let t = INDENT.get().map(|a| *a);
    match t {
        None => { INDENT.replace(Some(0)); }
        Some(i) => { INDENT.replace(Some(i + 1)); }
    }
}

fn unindent() {
    let t = INDENT.get().map(|a| *a);
    match t {
        None => (),
        Some(i) => {
            if i > 0 { INDENT.replace(Some(i - 1)); }
            else { INDENT.replace(None); }
        }
    }
}


impl ScopeTime {
    pub fn new(desc: &str) -> ScopeTime {
        indent();
        ScopeTime {
            t: time::precise_time_ns(),
            desc: String::from_str(desc),
        }
    }
}

impl Drop for ScopeTime {
    fn drop(&mut self) {
        let t = time::precise_time_ns() - self.t;
        match INDENT.get() {
            Some(i) => {
                for _ in range(0, *i) { print!(" "); }
            }
            _ => ()
        };
        println!("{}: {} s", self.desc, (t as f64) / 1e6f64);
        unindent();
    }
}
