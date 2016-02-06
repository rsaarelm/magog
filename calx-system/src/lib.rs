use std::path;

/// Return the application data directory path for the current platform.
pub fn app_data_path(app_name: &str) -> path::PathBuf {
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
                Ok(mut p) => { p.pop(); p }
                // If couldn't get self exe path, just use the local relative path and
                // hope for the best.
                _ => path::Path::new(".").to_path_buf()
            }
        } else {
            path::Path::new(
                &format!("{}\\{}", env::var("APPDATA").unwrap(), app_name))
            .to_path_buf()
        }
    } else if cfg!(macos) {
        path::Path::new(
            &format!("{}/Library/Application Support/{}",
                    env::var("HOME").unwrap(), app_name))
        .to_path_buf()
    } else {
        path::Path::new(
            &format!("{}/.config/{}", env::var("HOME").unwrap(), app_name))
        .to_path_buf()
    }
}
