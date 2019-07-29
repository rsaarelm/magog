use calx::tiled;
use euclid::point2;
use serde_json;
use std::io::{self, Read};
use std::iter::FromIterator;
use structopt;
use structopt::StructOpt;
use world::Sector;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(
        name = "sectors",
        about = "Add sector grid visualization layer to a map from stdin"
    )]
    Sectors {
        #[structopt(
            short = "w",
            long = "width",
            default_value = "8",
            help = "Width in sectors"
        )]
        width: u32,

        #[structopt(
            short = "h",
            long = "height",
            default_value = "8",
            help = "Height in sectors"
        )]
        height: u32,
    },
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Sectors { width, height } => {
            // Deserialize JSON map from stdin
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            let mut map: tiled::Map = serde_json::from_str(&buf)?;

            let mut checkerboard = Vec::new();
            for y in 0..(height as i16) {
                for x in 0..(width as i16) {
                    if (x + y) % 2 != 0 {
                        continue;
                    }
                    let sec = Sector::new(x, y, 0);
                    for loc in sec.iter() {
                        checkerboard.push(point2(loc.x as i32, loc.y as i32));
                    }
                }
            }

            let layer = tiled::Layer::TileLayer {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                opacity: 0.2,
                name: "Sector grid".to_string(),
                id: 123,
                visible: true,
                data: None,
                chunks: Some(tiled::ChunkMap::from_iter(
                    checkerboard.into_iter().map(|p| (p, 2)),
                )),
            };
            map.layers.push(layer);

            // Print out the modified map
            println!("{}", serde_json::to_string(&map)?);
        }
    }
    Ok(())
}
