use std::io::fs;
use std::io;
use std::os;
use std::io::{Process};

pub fn main() {
    for i in fs::readdir(&os::getcwd()).unwrap().iter() {
        let stat = i.stat().unwrap();
        if stat.kind == io::TypeFile && (stat.perm & io::UserExecute != 0) {
            let name = i.filename_str().unwrap();
            if name.starts_with("test_") {
                let mut run = Process::new(i.as_str().unwrap(), &[]).unwrap();
                let ret = run.wait();
                if !ret.success() {
                    println!("Unit test '{}' failed.", name);
                    os::set_exit_status(1);
                    return;
                }
            }
        }
    }
}
