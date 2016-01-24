//! Sprite display demo

extern crate image;
#[macro_use]
extern crate calx;

use calx::{V2, V3, color_key, Projection, Rect, ImageStore, IndexCache, noise};
use calx::color::*;
use calx::backend::{CanvasBuilder, WindowBuilder, CanvasUtil, Image, Event,
                    Key};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Brush {
    Grass,
    Dirt,
    Brick,
    Guy1,
    Guy2,
}

cache_key!(Brush);

fn build_sprites(builder: &mut CanvasBuilder) -> IndexCache<Brush, Image> {
    use Brush::*;

    let mut sprite_sheet =
        color_key(&image::load_from_memory(include_bytes!("assets/iso.png"))
                       .unwrap(),
                  CYAN);
    let mut ret = IndexCache::new();

    let keys = vec![Grass, Dirt, Brick, Guy1, Guy2];
    for (k, img) in keys.into_iter().zip(builder.batch_add(V2(-16, -24),
                                                           V2(32, 40),
                                                           &mut sprite_sheet)) {
        ret.insert(k, img);
    }
    ret
}

struct Sprite {
    pub bounds: (V3<f32>, V3<f32>),
    pub brush: Brush,
    // Sort key, pos projected along camera vector.
    pub key: f32,
}

impl Sprite {
    pub fn new(pos: V3<f32>, height: u32, brush: Brush) -> Sprite {
        Sprite {
            bounds: (pos, pos + V3(1.0, 1.0, height as f32 / 2.0)),
            brush: brush,
            key: pos.dot(V3(1.0, 1.0, 1.0)),
        }
    }
}

fn heightmap(cell: V2<f32>) -> u32 {
    let n = (cell.0 as i32) + 3329 * (cell.1 as i32);
    ((noise(n) + 1.0) * 1.5).round() as u32 + 1
}

fn gen_sprites(cell: V2<f32>) -> Vec<Sprite> {
    let mut offset = V3(cell.0, cell.1, 0.0);

    let mut ret = Vec::new();
    for _ in 0..heightmap(cell) {
        ret.push(Sprite::new(offset, 1, Brush::Dirt));
        offset.2 += 0.5;
    }
    let top = ret.len() - 1;
    ret[top].brush = Brush::Grass;

    ret
}

fn main() {
    let screen_rect = Rect(V2(0.0f32, 0.0f32), V2(640.0f32, 360.0f32));
    let screen_rect = screen_rect - screen_rect.dim() / 2.0;

    let window = WindowBuilder::new()
                     .set_size((screen_rect.1).0 as u32,
                               (screen_rect.1).1 as u32)
                     .build();
    let mut builder = CanvasBuilder::new();
    let mut player_x = 20.0;
    let mut player_y = 0.0;
    let cache = build_sprites(&mut builder);
    let mut ctx = builder.build(window);

    loop {
        let proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0))
                       .unwrap()
                       .world_offset(V2(-player_x, -player_y));

        let mut sprites = Vec::new();
        for pt in proj.inv_project_rectangle(&screen_rect).iter() {
            sprites.extend(gen_sprites(pt).into_iter());
        }
        sprites.push(Sprite::new(V3(player_x,
                                    player_y,
                                    heightmap(V2(player_x, player_y)) as f32 /
                                    2.0),
                                 3,
                                 Brush::Guy1));

        sprites.sort_by(|x, y| x.key.partial_cmp(&y.key).unwrap());

        for spr in sprites.iter() {
            let draw_pos = proj.project(V2((spr.bounds.0).0,
                                           (spr.bounds.0).1)) +
                           V2(0.0, -16.0 * (spr.bounds.0).2) +
                           screen_rect.dim() / 2.0;
            ctx.draw_image(*cache.get(spr.brush).unwrap(),
                           draw_pos,
                           0.5,
                           WHITE,
                           BLACK);
        }

        for event in ctx.events().into_iter() {
            match event {
                Event::Quit => {
                    return;
                }

                Event::KeyPress(Key::Escape) => {
                    return;
                }

                Event::KeyPress(Key::F12) => {
                    ctx.save_screenshot(&"isoworld");
                }

                Event::KeyPress(k) => {
                    match k {
                        Key::A => {
                            player_x -= 1.0;
                        }
                        Key::D => {
                            player_x += 1.0;
                        }
                        Key::W => {
                            player_y -= 1.0;
                        }
                        Key::S => {
                            player_y += 1.0;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        ctx.end_frame();
    }
}
