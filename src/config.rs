use std::default::Default;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use getopts::{Options};
use toml::{self, Value};
use calx::backend::{CanvasMagnify};
use calx::vorud::{Vorud, FromVorud};
use super::app_data_path;

thread_local!(pub static _CONFIG: Config = Default::default());

#[derive(Copy, Clone, Debug)]
pub struct Config {
    /// The player will move to the side when bumping against a wall.
    pub wall_sliding: bool,
    /// How to scale the graphics when making the window larger.
    pub magnify_mode: CanvasMagnify,
    /// Fixed world random number generator seed.
    pub rng_seed: Option<u32>,
    /// Start in fullscreen mode
    pub fullscreen: bool,
    /// Show FPS counter
    pub show_fps: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            wall_sliding: true,
            magnify_mode: CanvasMagnify::PixelPerfect,
            rng_seed: None,
            fullscreen: false,
            show_fps: false,
        }
    }
}

impl Config {
    /// Parse command line options and apply them to the config object. Return
    /// an error containing a usage string if the parsing fails.
    ///
    /// Returning a string with Ok indicates that the string should be printed
    /// and the program should then exit.
    pub fn parse_args<T: Iterator<Item=String>>(&mut self, args: T) -> Result<Option<String>, String> {
        let mut opts = Options::new();
        opts.optflag("", "no-wall-sliding", "Don't move diagonally along obstacles when walking into them");
        opts.optflag("h", "help", "Display this message");
        opts.optflag("V", "version", "Print version info and exit");
        opts.optopt("", "magnify-mode", "How to filter magnified graphics. MODE = pixel | nearest | smooth", "MODE");
        opts.optopt("", "seed", "World generation seed", "VORUD");
        opts.optflag("", "fullscreen", "Run in fullscreen mode");
        opts.optflag("", "fps", "Show FPS counter");

        let args: Vec<String> = args.collect();

        let parse = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                return Err(usage_error(f.to_string(), &opts));
            }
        };

        if parse.opt_present("help") {
            return Ok(Some(format!("{}", opts.usage("Usage: magog [options]"))));
        }

        if parse.opt_present("version") {
            return Ok(Some(format!("Magog v{}",
                                ::version())));
        }

        if parse.opt_present("no-wall-sliding") {
            self.wall_sliding = false;
        }

        if parse.opt_present("fullscreen") {
            self.fullscreen = true;
        }

        if parse.opt_present("fps") {
            self.show_fps = true;
        }

        if let Some(x) = parse.opt_str("magnify-mode") {
            match &x[..] {
                "pixel" => { self.magnify_mode = CanvasMagnify::PixelPerfect; }
                "nearest" => { self.magnify_mode = CanvasMagnify::Nearest; }
                "smooth" => { self.magnify_mode = CanvasMagnify::Smooth; }
                err => {
                    return Err(usage_error(format!("Unknown magnify mode '{}'", err), &opts));
                }
            }
        }

        if let Some(seed) = parse.opt_str("seed") {
            // XXX: Could this be written to use the try! macro?
            let err = Err(usage_error(format!("Invalid seed value '{}'", seed), &opts));
            if let Ok(vorud) = Vorud::new(seed.clone()) {
                if let Ok(val) = FromVorud::from_vorud(&vorud) {
                    self.rng_seed = Some(val);
                } else {
                    return err;
                }
            } else {
                return err;
            }
        }

        return Ok(None);

        fn usage_error(msg: String, opts: &Options) -> String {
            format!("{}\n{}",
                    msg, opts.usage("Usage:\n    magog [options]"))
        }
    }

    pub fn file_path(&self) -> PathBuf {
        let mut ret = app_data_path();
        ret.push("config.toml");
        ret
    }

    /// Output the default config file.
    pub fn default_file(&self) -> String {
        // TODO: Make this data-driven, have option metadata that specifies
        // the help string and what to print out.
        format!(
r#"# Move diagonally along obstacles when walking into them
wall-sliding = true

# How to filter magnified graphics. pixel | nearest | smooth
magnify-mode = "pixel"

# Start the game in fullscreen instead of windowed mode
fullscreen = false

# Display FPS counter
fps = false
"#)
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        // TODO: Error handling.

        // TODO: This should be data-driven somehow from a common config
        // metadata object instead of relying on the error-prone retyped
        // schema in the match segment.
        let mut file = File::open(path).unwrap();
        let mut toml = String::new();
        file.read_to_string(&mut toml).unwrap();

        let mut parser = toml::Parser::new(&toml);

        let settings = parser.parse();

        if settings.is_none() {
            let mut err = String::new();
            for e in parser.errors.iter() {
                err.push_str(&format!("{}", e));
            }
            return Err(err);
        }

        let settings = settings.unwrap();

        // TODO: Error message instead of silent no-op if the type of the
        // value isn't what we expect.
        if let Some(&Value::Boolean(b)) = settings.get("wall-sliding") {
            self.wall_sliding = b;
        }

        if let Some(&Value::String(ref mag)) = settings.get("magnify-mode") {
            match &mag[..] {
                "pixel" => { self.magnify_mode = CanvasMagnify::PixelPerfect; }
                "nearest" => { self.magnify_mode = CanvasMagnify::Nearest; }
                "smooth" => { self.magnify_mode = CanvasMagnify::Smooth; }
                _ => { return Err(format!("Bad magnify mode '{}'", mag)); }
            };
        }

        if let Some(&Value::Boolean(b)) = settings.get("fullscreen") {
            self.fullscreen = b;
        }

        if let Some(&Value::Boolean(b)) = settings.get("fps") {
            self.show_fps = b;
        }

        Ok(())
    }
}
