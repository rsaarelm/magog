///! Generate a base bitmap for drawing an overland map with a paint program.

extern crate world;
extern crate image;

use image::Pixel;
use world::{Sector, Location};

const COLS: i16 = 10;
const ROWS: i16 = 10;

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

    let mut buf = image::ImageBuffer::new((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32);

    for (x, y, p) in buf.enumerate_pixels_mut() {
        let loc = Location::new(x as i16 + min_x, y as i16 + min_y, 0);
        let sec = loc.sector();
        if sec.x < 0 || sec.y < 0 || sec.x >= COLS || sec.y >= ROWS {
            *p = image::Rgb::from_channels(0x00, 0x00, 0x00, 0xff);
            continue;
        }
        if (sec.x + sec.y) % 2 == 0 {
            *p = image::Rgb::from_channels(0x00, 0xaa, 0xaa, 0xff);
        } else {
            *p = image::Rgb::from_channels(0x88, 0x88, 0x88, 0xff);
        }
    }

    image::save_buffer(
        "overland_base.png",
        &buf,
        buf.width(),
        buf.height(),
        image::ColorType::RGB(8),
    ).unwrap();
}
