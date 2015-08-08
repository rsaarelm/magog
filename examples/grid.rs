/*! Interactive grid projection demo */

extern crate calx;

use calx::{noise, color, V2, Rgba, Rect, Projection, Anchor};
use calx::backend::{WindowBuilder, CanvasBuilder, Canvas, CanvasUtil, Event, Key, RenderTarget};
use calx::backend::mesh::Vertex;

static PROJECTIONS: [(V2<f32>, V2<f32>); 3] = [
    (V2(16.0, 8.0), V2(-16.0, 8.0)),  // Isometric
    (V2(16.0, 0.0), V2(0.0, 16.0)),   // Topdown
    (V2(16.0, 0.0), V2(-6.0, 12.0)),  // Cavalier
];

fn draw_cell(ctx: &mut Canvas, proj: &Projection, cell: V2<i32>, dim: bool) {
    let n = cell.0 + 3329 * cell.1;
    // Procedural color.
    let mut color = Rgba::new(
        noise(n * 3) / 2.0 + 0.5,
        noise(n * 3 + 1) / 2.0 + 0.5,
        noise(n * 3 + 2) / 2.0 + 0.5,
        1.0);
    if dim {
        color = color.to_monochrome();
    }

    let tex = ctx.solid_tex_coord();

    // Project world-space cell base into screen space.
    let base = Rect(cell.map(|x| x as f32), V2(1.0, 1.0));

    let mut vertices = Vec::new();
    for &p in [Anchor::TopLeft, Anchor::TopRight, Anchor::BottomRight, Anchor::BottomLeft].iter() {
        vertices.push(Vertex::new(proj.project(base.point(p)), 0.5, tex, color, color::BLACK));
    }

    ctx.add_mesh(vertices, vec![[0, 1, 2], [0, 2, 3]]);
}

fn main() {
    let mut projection = 0;
    let scroll_speed = 2f32;
    let mut mouse_pos = V2(-1i32, -1i32);
    let mut screen_offset = V2(0.0f32, 0.0f32);
    let mut scroll_delta = V2(0.0f32, 0.0f32);
    let mut ctx = CanvasBuilder::new().build(WindowBuilder::new().build());

    let screen_rect = Rect(V2(0.0f32, 0.0f32), V2(640.0f32, 360.0f32));

    loop {
        screen_offset = screen_offset - scroll_delta;
        let proj = Projection::new(PROJECTIONS[projection].0, PROJECTIONS[projection].1)
            .unwrap()
            .view_offset(screen_offset);

        let mouse_rect = Rect(mouse_pos.map(|x| x as f32) - V2(160.0, 90.0), V2(320.0, 180.0));
        let mouse_cell = proj.inv_project(mouse_pos.map(|x| x as f32)).map(|x| x.floor() as i32);

        for pt in proj.inv_project_rectangle(&screen_rect).iter() {
            let cell = pt.map(|x| x as i32);
            if cell != mouse_cell { draw_cell(&mut ctx, &proj, cell, true); }
        }

        for pt in proj.inv_project_rectangle(&mouse_rect).iter() {
            let cell = pt.map(|x| x as i32);
            if cell != mouse_cell { draw_cell(&mut ctx, &proj, cell, false); }
        }

        ctx.draw_rect(&mouse_rect, 0.5, color::CYAN);

        for event in ctx.events().into_iter() {
            match event {
                Event::Quit => { return; }
                Event::RenderFrame => {
                }

                Event::KeyPress(Key::Escape) => { return; }

                Event::KeyPress(Key::Tab) => {
                    projection += 1;
                    projection %= PROJECTIONS.len();
                }

                Event::KeyPress(Key::F12) => {
                    ctx.save_screenshot(&"grid");
                }

                Event::KeyPress(k) => {
                    match k {
                        Key::A => { scroll_delta.0 = -1.0 * scroll_speed; }
                        Key::D => { scroll_delta.0 =  1.0 * scroll_speed; }
                        Key::W => { scroll_delta.1 = -1.0 * scroll_speed; }
                        Key::S => { scroll_delta.1 =  1.0 * scroll_speed; }
                        _ => {}
                    }
                }

                Event::KeyRelease(k) => {
                    match k {
                        Key::A => { scroll_delta.0 = 0.0; }
                        Key::D => { scroll_delta.0 = 0.0; }
                        Key::W => { scroll_delta.1 = 0.0; }
                        Key::S => { scroll_delta.1 = 0.0; }
                        _ => {}
                    }
                }
                Event::MouseMove(pos) => {
                    mouse_pos = pos.map(|x| x as i32);
                }
                _ => {}
            }
        }

        ctx.end_frame();
    }
}
