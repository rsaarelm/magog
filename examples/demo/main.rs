extern crate calx;

use calx::color;
use calx::{V2};
use calx::event;

fn main() {
    let mut t = 0i;

    for evt in calx::Canvas::new().run() {
        match evt {
            event::Render(ctx) => {
                ctx.clear(&calx::Rgb::new(t as u8, 0, 0));
                let img = ctx.font_image('F').unwrap();
                ctx.draw_image(V2(1, 9), 0.4, img, &color::ORANGE);
                ctx.draw_line(V2(10, 10), V2(100, 50), 0.5, 3f32, &color::YELLOW);
                t += 1;
            }
            event::KeyPressed(calx::key::KeyEscape) => {
                return;
            }
            _ => ()
        }
    }
}
