extern crate calx;

use calx::color;
use calx::{V2};
use calx::event;

fn main() {
    let mut t = 0i;

    for evt in calx::Canvas::new().run() {
        match evt {
            event::Render(ctx) => {
                let img = ctx.font_image('@').unwrap();

                ctx.clear(&calx::Rgb::new(t as u8, 0, 0));
                for y in range(0, 360/8) {
                    for x in range(0, 640/8) {
                        ctx.draw_image(V2(x * 8, y * 8), 0.4, img, &color::ORANGE);
                    }
                }
                t += 1;
            }
            event::KeyPressed(calx::key::KeyEscape) => {
                return;
            }
            _ => ()
        }
    }
}
