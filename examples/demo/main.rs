extern crate calx;

use calx::color;
use calx::{V2, Rgba, Fonter, CanvasUtil};
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
                let center = V2(320, 180);
                let offset = V2(
                    ((t as f32 / 16.0).cos() * 128.0) as int,
                    ((t as f32 / 16.0).sin() * 128.0) as int);

                ctx.draw_line(3, center, center + offset, 0.3, &Rgba::new(0, 255, 255, 128));

                let fps = 1.0 / ctx.render_duration;
                let _ = write!(&mut ctx.text_writer(V2(0, 8), 0.1, color::LIGHTGREEN)
                               .set_border(color::BLACK),
                    "FPS {:.0}", fps);

                t += 1;
            }
            event::KeyPressed(calx::key::KeyEscape) => {
                return;
            }
            _ => ()
        }
    }
}
