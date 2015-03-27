use getopts::{Options};
use std::default::Default;
use calx::backend::{CanvasMagnify};

thread_local!(pub static CONFIG: Config = Default::default());

#[derive(Debug)]
pub struct Config {
    /// The player will move to the side when bumping against a wall.
    pub wall_sliding: bool,
    /// How to scale the graphics when making the window larger.
    pub magnify_mode: CanvasMagnify,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            wall_sliding: true,
            magnify_mode: CanvasMagnify::PixelPerfect,
        }
    }
}

impl Config {
    /// Parse command line options and apply them to the config object. Return
    /// an error containing a usage string if the parsing fails.
    pub fn parse_args<T: Iterator<Item=String>>(&mut self, args: T) -> Result<(), String> {
        let mut opts = Options::new();
        opts.optflag("", "no-wall-sliding", "Don't move diagonally along obstacles when walking into them");
        opts.optopt("", "magnify-mode", "How to filter magnified graphics. MODE = pixel | nearest | smooth", "MODE");

        let args: Vec<String> = args.collect();

        let parse = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                return Err(usage(f.to_string(), &opts));
            }
        };

        if parse.opt_present("no-wall-sliding") {
            self.wall_sliding = false;
        }

        match parse.opt_str("magnify-mode") {
            Some(x) => match x.as_slice() {
                "pixel" => { self.magnify_mode = CanvasMagnify::PixelPerfect; }
                "nearest" => { self.magnify_mode = CanvasMagnify::Nearest; }
                "smooth" => { self.magnify_mode = CanvasMagnify::Smooth; }
                err => {
                    return Err(usage(format!("Unknown magnify mode '{}'", err), &opts));
                }
            },
            _ => {}
        }

        return Ok(());

        fn usage(msg: String, opts: &Options) -> String {
            format!("{}\n\n{}",
                    msg, opts.usage("Usage:\n    magog [options]"))
        }
    }
}
