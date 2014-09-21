extern crate calx;

fn main() {
    let mut t = 0i;

    for evt in calx::Canvas::new().run() {
        match evt {
            calx::Render(ctx) => {
                ctx.clear(&calx::Rgb::new(t as u8, 0, 0));
                ctx.draw_test();
                t += 1;
            }
            calx::KeyPressed(calx::key::KeyEscape) => {
                return;
            }
            _ => ()
        }
    }
}
