///! Generate a base bitmap for drawing an overland map with a paint program.

extern crate calx;
extern crate world;
extern crate image;

use image::Pixel;
use world::{Sector, Location, Terrain};
use calx::hex_disc;

const COLS: i16 = 8;
const ROWS: i16 = 12;

fn overland_locs() -> Vec<Location> {
    let mut ret = Vec::new();
    for y in 0..ROWS {
        for x in 0..COLS {
            let sec = Sector::new(x, y, 0);
            for loc in sec.iter() {
                ret.push(loc);
            }
        }
    }
    ret
}

fn valid_sector(sec: Sector) -> bool {
    sec.x >= 0 && sec.y >= 0 && sec.x < COLS && sec.y < ROWS
}

fn main() {
    let locs = overland_locs();
    let (mut min_x, mut min_y) = locs.iter().fold(
        (0, 0),
        |(x, y), loc| (x.min(loc.x), y.min(loc.y)),
    );
    min_x -= 1;
    min_y -= 1;

    let (max_x, max_y) = locs.iter().fold(
        (0, 0),
        |(x, y), loc| (x.max(loc.x), y.max(loc.y)),
    );
    println!("{} {} - {} {}", min_x, min_y, max_x, max_y);

    let mut buf = image::ImageBuffer::new((max_x - min_x + 2) as u32, (max_y - min_y + 2) as u32);

    for (x, y, p) in buf.enumerate_pixels_mut() {
        let loc = Location::new(x as i16 + min_x, y as i16 + min_y, 0);
        let sec = loc.sector();
        let terrain = if valid_sector(sec) {
            Terrain::Grass
        } else if hex_disc(loc, 1).any(|loc| valid_sector(loc.sector())) {
            // Not a valid sector, but touching one.
            Terrain::Water
        } else {
            *p = image::Rgb::from_channels(0x00, 0x00, 0x00, 0xff);
            continue;
        };

        let color = if (sec.x + sec.y) % 2 == 0 {
            terrain.color()
        } else {
            terrain.dark_color()
        };

        *p = image::Rgb::from_channels(color.r, color.g, color.b, 0xff);
    }

    // XXX: Hacky way to force terrain colors into image palette
    // Didn't find direct indexed palette manipulation in piston-image.
    for (x, t) in Terrain::iter().filter(|t| t.is_regular()).enumerate() {
        let light = t.color();
        let dark = t.dark_color();
        buf.put_pixel(x as u32, 0, image::Rgb::from_channels(light.r, light.g, light.b, 0xff));
        buf.put_pixel(x as u32, 1, image::Rgb::from_channels(dark.r, dark.g, dark.b, 0xff));
    }

    image::save_buffer(
        "overland_base.png",
        &buf,
        buf.width(),
        buf.height(),
        image::ColorType::RGB(8),
    ).unwrap();
}
