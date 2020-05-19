use calx::tiled;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(help = "Tiled JSON map")]
    map: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let file = File::open(opt.map).unwrap();
    let map: tiled::Map = serde_json::from_reader(BufReader::new(file)).unwrap();
    let map = world::WorldData::try_from(map).unwrap();
    print!("{}", map);
}
