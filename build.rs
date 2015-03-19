// Generate the git version string.

use std::io::prelude::*;
use std::process::{Command, Output};
use std::fs::{File};

fn git_version() -> String {
    match Command::new("git")
        .arg("log").arg("--pretty=format:%h").arg("-1").output() {
        Ok(Output { status: exit, stdout: out, stderr: err }) => {
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

fn rustc_version() -> String {
    match Command::new("rustc")
        .arg("--version").output() {
        Ok(Output { status: exit, stdout: out, stderr: err }) => {
            if exit.success() {
                return String::from_utf8(out).unwrap();
            } else {
                println!("Error getting rustc version: {}", String::from_utf8(err).unwrap());
            }
        }
        Err(err) => {
            println!("Error getting rustc version: {}", err);
        }
    }
    return "unknown".to_string();
}

fn open(path: &str) -> File {
    match File::create(path) {
        Ok(f) => f, Err(e) => panic!("file error: {}", e),
    }
}

pub fn main() {
    // Write the current Git HEAD hash into the version file.
    write!(&mut open("src/git_hash.inc"), "{}", git_version()).unwrap();
    write!(&mut open("rustc_version.txt"), "{}", rustc_version()).unwrap();
}
