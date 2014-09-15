extern crate blot;

fn main() {
    let mut t = 0i;

    for evt in blot::Canvas::new().run() {
        match evt {
            blot::Render(ctx) => {
                ctx.clear([0.0, (t as f32) / 256f32 % 1.0, 0.0, 1.0]);
                ctx.draw_test();
                t += 1;
            }
            blot::Input(e) => {
                println!("Input event {}", e);
            }
        }
    }
}
