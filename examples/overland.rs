///! Generate a base bitmap for drawing an overland map with a paint program.
extern crate calx;
extern crate euclid;
extern crate image;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate world;

use calx::{hex_disc, CellVector, FromPrefab, IntoPrefab, MinimapSpace, ProjectedImage, SRgba};
use euclid::vec2;
use image::{GenericImage, Pixel, SubImage};
use std::collections::HashMap;
use structopt::StructOpt;
use world::{Location, Sector, Terrain};

type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

type Prefab<T> = HashMap<CellVector, T>;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "generate", help = "Generate a blank overland map image")]
    Generate {
        #[structopt(long = "minimap", help = "Use minimap projection")]
        minimap: bool,

        #[structopt(short = "w", long = "width", default_value = "8", help = "Width in sectors")]
        width: u32,

        #[structopt(short = "h", long = "height", default_value = "9", help = "Height in sectors")]
        height: u32,

        #[structopt(help = "Output PNG file")]
        output: String,
    },

    #[structopt(name = "convert",
                help = "Convert map from one projection to another and normalize the checkerboard pattern")]
    Convert {
        #[structopt(long = "input-minimap", help = "Input file has minimap projection")]
        input_minimap: bool,

        #[structopt(long = "output-minimap", help = "Use minimap projection in output file")]
        output_minimap: bool,

        #[structopt(help = "Input file")]
        input: String,

        #[structopt(help = "Output file (if different from input)")]
        output: Option<String>,
    },
}

fn default_map(width: u32, height: u32) -> Prefab<Terrain> {
    fn p(loc: Location) -> CellVector { vec2(loc.x as i32, loc.y as i32) }

    let mut terrain = HashMap::new();

    for &loc in &overland_locs(width, height) {
        for edge in hex_disc(loc, 1) {
            // Ensure land is surrounded by water past the sector edges.
            terrain.entry(p(edge)).or_insert(Terrain::Water);
        }
        terrain.insert(p(loc), Terrain::Grass);
    }

    terrain
}

fn dark(color: SRgba) -> SRgba {
    let mut color = color;
    color.r &= !0xF;
    color.g &= !0xF;
    color.b &= !0xF;
    color
}

fn light(color: SRgba) -> SRgba {
    let mut color = dark(color);
    color.r += 0xF;
    color.g += 0xF;
    color.b += 0xF;
    color
}

fn checkerboard((pos, color): (CellVector, SRgba)) -> (CellVector, SRgba) {
    let sec = Location::new(pos.x as i16, pos.y as i16, 0).sector();
    let is_dark = (sec.x + sec.y) % 2 != 0;
    (pos, if is_dark { dark(color) } else { light(color) })
}

fn terrain_to_color((pos, terrain): (CellVector, Terrain)) -> (CellVector, SRgba) {
    checkerboard((pos, terrain.color()))
}

fn overland_locs(width: u32, height: u32) -> Vec<Location> {
    let mut ret = Vec::new();
    for y in 0..(height as i16) {
        for x in 0..(width as i16) {
            let sec = Sector::new(x, y, 0);
            for loc in sec.iter() {
                ret.push(loc);
            }
        }
    }
    ret
}

fn save(prefab: Prefab<SRgba>, is_minimap: bool, output_path: String) {
    let image: ImageBuffer = if is_minimap {
        let p: ProjectedImage<ImageBuffer, MinimapSpace> = FromPrefab::from_prefab(&prefab);
        p.image
    } else {
        FromPrefab::from_prefab(&prefab)
    };

    // Impose palette
    let mut result = ImageBuffer::new(image.width(), image.height() + 1);
    result.copy_from(&image, 0, 0);

    for (x, t) in Terrain::iter().filter(|t| t.is_regular()).enumerate() {
        let light = light(t.color());
        let dark = dark(t.color());
        let y = result.height() - 1;
        result.put_pixel(
            x as u32 * 2,
            y,
            image::Rgba::from_channels(light.r, light.g, light.b, 0xff),
        );
        result.put_pixel(
            x as u32 * 2 + 1,
            y,
            image::Rgba::from_channels(dark.r, dark.g, dark.b, 0xff),
        );
    }

    image::save_buffer(
        output_path,
        &result,
        result.width(),
        result.height(),
        image::ColorType::RGBA(8),
    ).unwrap();
}

fn generate(width: u32, height: u32, is_minimap: bool, output_path: String) {
    let prefab: Prefab<SRgba> = default_map(width, height)
        .into_iter()
        .map(terrain_to_color)
        .collect();

    save(prefab, is_minimap, output_path);
}

fn convert(
    input_path: String,
    input_is_minimap: bool,
    output_path: Option<String>,
    output_is_minimap: bool,
) {
    let mut input =
        image::open(input_path.clone()).expect(&format!("Unable to load '{}'", input_path.clone()));
    // Slice off the bottom row containing palette (h - 1).
    let (w, h) = (input.width(), input.height());
    let input_map = SubImage::new(&mut input, 0, 0, w, h - 1);

    let prefab: Prefab<SRgba> = if input_is_minimap {
        let p: ProjectedImage<_, MinimapSpace> = ProjectedImage::new(input_map);
        p.into_prefab().expect("Bad map image")
    } else {
        input_map.into_prefab().expect("Bad map image")
    };

    // Impose sector checkerboard.
    let prefab: Prefab<SRgba> = prefab.into_iter().map(checkerboard).collect();

    save(prefab, output_is_minimap, output_path.unwrap_or(input_path));
}

fn main() {
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Generate {
            width,
            height,
            minimap,
            output,
        } => generate(width, height, minimap, output),
        Command::Convert {
            input,
            input_minimap,
            output,
            output_minimap,
        } => convert(input, input_minimap, output, output_minimap),
    }
}
