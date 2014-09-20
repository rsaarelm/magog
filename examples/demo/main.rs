extern crate calx;

fn main() {
    let mut t = 0i;

    for evt in calx::Canvas::new().run() {
        match evt {
            calx::Render(ctx) => {
                ctx.clear([0.0, (t as f32) / 256f32 % 1.0, 0.0, 1.0]);
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
