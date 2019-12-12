use euclid::vec2;
use world::Location;

// Use smooth noise to wobblify the edges

const HEX_SIZE: i32 = 20;
//const NOISE: noise::OpenSimplex = noise::OpenSimplex::new();

fn main() {
    for r in 0..60 {
        for c in 0..120i32 {
            if (c - r).rem_euclid(2) == 1 {
                print!(" ");
                continue;
            }
            let (x, y) = ((c + r) / 2 - 50, r - 30);
            let offset = Location::new(x as i16, y as i16, 0).terrain_cell_displacement();

            let r = calx::HexGeom::hex_dist(&vec2(x + offset.x, y + offset.y));
            if r < HEX_SIZE {
                print!("*");
            } else if r == HEX_SIZE {
                print!(".");
            } else {
                print!(" ");
            }
        }
        println!();
    }
}
