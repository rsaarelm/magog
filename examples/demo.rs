extern crate time;
extern crate calx;

use std::ascii::AsciiExt;
use calx::{color, V2, Rgba, Rect};
use calx::backend::{CanvasBuilder, Key, Event, Fonter, CanvasUtil};
use calx::{lerp};

fn main() {
    let mut t = 0i32;
    let mut mouse_pos = V2(-1i32, -1i32);
    let pangrams = vec![
        "the quick brown fox jumps over the lazy dog",
        "the five boxing wizards jump quickly",
        "five quacking zephyrs jolt my wax bed",
        "jackdaws love my big sphinx of quartz",
        "the quick onyx goblin jumps over the lazy dwarf",
        "who packed five dozen old quart jugs in my box",
        "heavy boxes perform quick waltzes and jigs",
        "fix problem quickly with galvanized jets",
        "pack my red box with five dozen quality jugs",
        "why shouldn't a quixotic kazakh vampire jog barefoot",
        "have a pick: twenty-six letters - no forcing a jumbled quiz",
        "crazy frederick bought many very exquisite opal jewels",
        "grumpy wizards make toxic brew for the evil queen and jack",
        "just keep examining every low bid quoted for zinc etchings",
        "sylvia wagt quick den jux bei pforzheim",
        "franz jagt im komplett verwahrlosten taxi quer durch",
        "sic fugiens, dux, zelotypos, quam karus haberis",
        ".o'i mu xagji sofybakni cu zvati le purdi",
    ];
    let mut ctx = CanvasBuilder::new().build();
    loop {
        // Change pangram every 10 seconds.
        let pangram_idx = (time::precise_time_s() / 10.0) as usize % pangrams.len();
        match ctx.next_event() {
            Event::Quit => { break; }
            Event::RenderFrame => {
                let img = ctx.font_image('@').unwrap();

                ctx.clear();
                for y in 0i32..(360/8) {
                    for x in 0i32..(640/8) {
                        let col = if Rect(V2(x * 8, y * 8), V2(8, 8)).contains(&mouse_pos) {
                            0xFFFFFFFF } else { 0x442200FF };
                        ctx.draw_image(img, V2(x as f32 * 8.0, y as f32 * 8.0 + 8.0),
                            0.4, col, color::BLACK);
                    }
                }

                // These should be clipped off the screen.
                let img = ctx.font_image('#').unwrap();
                ctx.draw_image(img, V2(316.0, 0.0), 0.4, color::GREEN, color::BLACK);
                ctx.draw_image(img, V2(316.0, 368.0), 0.4, color::GREEN, color::BLACK);
                ctx.draw_image(img, V2(-8.0, 184.0), 0.4, color::GREEN, color::BLACK);
                ctx.draw_image(img, V2(640.0, 184.0), 0.4, color::GREEN, color::BLACK);

                let center = V2(320.0, 180.0);
                let offset = V2(
                    ((t as f32 / 160.0).cos() * 128.0),
                    ((t as f32 / 160.0).sin() * 128.0));

                ctx.draw_line(3.0, center, center + offset, 0.3, Rgba::new(0.0, 1.0, 1.0, 0.1));

                let fps = 1.0 / ctx.render_duration;
                {
                    let mut fonter = Fonter::new(&mut ctx)
                        .color(lerp(color::RED, color::GREEN, (t as f32 / 100.0) % 1.0))
                        .border(color::BLACK)
                        .layer(0.1)
                        .text(format!("FPS {:.0}\n", fps))
                        .text(format!("{}\n", pangrams[pangram_idx].to_string()
                                              .to_ascii_uppercase()))
                        .text(format!("{}\n", pangrams[pangram_idx]))
                        .text(format!("!\"#$%&'()*+,-./\n"))
                        .text(format!("1234567890:;<=>?\n"))
                        .text(format!("[\\]^_`{{|}}~\n"));
                    fonter.draw(V2(0.0, 0.0));
                }

                t += 1;
            }
            Event::KeyPressed(Key::Escape) => {
                return;
            }
            Event::KeyPressed(k) => {
                println!("Pressed {:?}", k);
            }
            Event::Char(c) => {
                println!("Typed {:?}", c);
            }
            Event::MouseMoved((x, y)) => {
                mouse_pos = V2(x, y);
            }
            _ => ()
        }
    }
}
