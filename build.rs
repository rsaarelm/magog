#![feature(io, path)]

// Generate the git version string.

use std::old_io::process::{Command, ProcessOutput};
use std::old_io::{File};

fn get_version() -> String {
    match Command::new("git")
        .arg("log").arg("--pretty=format:%h").arg("-1").output() {
        Ok(ProcessOutput { status: exit, output: out, error: err }) => {
            if exit.success() {
                return String::from_utf8(out).unwrap();
            } else {
                println!("Error getting git version: {}", String::from_utf8(err).unwrap());
            }
        }
        Err(err) => {
            println!("Error getting git version: {}", err);
        }
    }
    return "unknown".to_string();
}

pub fn main() {
    // Write the current Git HEAD hash into the version file.
    let mut file = match File::create(&Path::new("src/version.inc")) {
        Ok(f) => f, Err(e) => panic!("file error: {}", e),
    };
    write!(&mut file, "{}", get_version()).unwrap();
}
