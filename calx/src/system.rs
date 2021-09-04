use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempdir::TempDir;

/**
 * Return seconds elapsed since epoch.
 */
pub fn precise_time_s() -> f64 {
    let duration = time::OffsetDateTime::now_utc() - time::OffsetDateTime::UNIX_EPOCH;
    duration.as_seconds_f64()
}

/// Return the application data directory path for the current platform.
pub fn app_data_path(app_name: &str) -> PathBuf {
    use std::env;
    // On Windows, a portable application is just an .exe the user downloads
    // and drops somewhere. The convention here is for a portable application
    // to add its files to wherever its exe file is. An installed application
    // uses an actual installer program and deploys its files to user data
    // directories.
    let is_portable_application = true;

    // TODO: Handle not having the expected env variables.
    if cfg!(windows) {
        if is_portable_application {
            match env::current_exe() {
                Ok(mut p) => {
                    p.pop();
                    p
                }
                // If couldn't get self exe path, just use the local relative path and
                // hope for the best.
                _ => Path::new(".").to_path_buf(),
            }
        } else {
            Path::new(&format!("{}\\{}", env::var("APPDATA").unwrap(), app_name)).to_path_buf()
        }
    } else if cfg!(macos) {
        Path::new(&format!(
            "{}/Library/Application Support/{}",
            env::var("HOME").unwrap(),
            app_name
        ))
        .to_path_buf()
    } else {
        Path::new(&format!(
            "{}/.config/{}",
            env::var("HOME").unwrap(),
            app_name
        ))
        .to_path_buf()
    }
}

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
        // if TIME_LOG.state() == LocalKeyState::Destroyed { return; }
        TIME_LOG.with(|a| {
            let mut n = 1;
            let contains = a.borrow().logs.contains_key(&name);
            if contains {
                let (old_n, total) = a.borrow().logs[&name];
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
        for (name, &(n, total)) in &self.logs {
            println!(
                "  {}:\t{:.3} s\t(avg. {:.3} s)",
                name,
                total,
                total / (n as f64)
            );
        }
    }
}

thread_local!(static TIME_LOG: RefCell<TimeLog> = RefCell::new(TimeLog::new()));

/// Debug object which prints the total time spent executing scopes it was in
/// after the program finishes running.
#[must_use]
pub struct TimeLogItem {
    name: String,
    begin: f64,
}

impl TimeLogItem {
    pub fn new(name: &str) -> TimeLogItem {
        TimeLogItem {
            name: name.to_string(),
            begin: precise_time_s(),
        }
    }
}

impl Drop for TimeLogItem {
    fn drop(&mut self) { TimeLog::log(self.name.clone(), precise_time_s() - self.begin); }
}

/// Save a timestamped screenshot to disk.
pub fn save_screenshot(
    basename: &str,
    shot: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tmpdir = TempDir::new("calx")?;
    let timestamp = (precise_time_s() * 100.0) as u64;
    let file = Path::new(&format!("{}-{}.png", basename, timestamp)).to_path_buf();
    let tmpfile = tmpdir.path().join(file.clone());
    image::save_buffer(
        &tmpfile,
        shot,
        shot.width(),
        shot.height(),
        image::ColorType::Rgb8,
    )?;

    Ok(fs::copy(&tmpfile, &file).map(|_| ())?)
}
