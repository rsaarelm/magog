use getopts::{Options};
use std::default::Default;
use calx::backend::{CanvasMagnify};
use calx::vorud::{Vorud, FromVorud};

thread_local!(pub static _CONFIG: Config = Default::default());

#[derive(Copy, Debug)]
pub struct Config {
    /// The player will move to the side when bumping against a wall.
    pub wall_sliding: bool,
    /// How to scale the graphics when making the window larger.
    pub magnify_mode: CanvasMagnify,
    /// Fixed world random number generator seed.
    pub rng_seed: Option<u32>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            wall_sliding: true,
            magnify_mode: CanvasMagnify::PixelPerfect,
            rng_seed: None,
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
        opts.optopt("", "seed", "World generation seed", "VORUD");

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

        if let Some(x) = parse.opt_str("magnify-mode") {
            match &x[..] {
                "pixel" => { self.magnify_mode = CanvasMagnify::PixelPerfect; }
                "nearest" => { self.magnify_mode = CanvasMagnify::Nearest; }
                "smooth" => { self.magnify_mode = CanvasMagnify::Smooth; }
                err => {
                    return Err(usage(format!("Unknown magnify mode '{}'", err), &opts));
                }
            }
        }

        if let Some(seed) = parse.opt_str("seed") {
            // XXX: Could this be written to use the try! macro?
            let err = Err(usage(format!("Invalid seed value '{}'", seed), &opts));
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

        return Ok(());

        fn usage(msg: String, opts: &Options) -> String {
            format!("{}\n\n{}",
                    msg, opts.usage("Usage:\n    magog [options]"))
        }
    }
}
