use euclid::{Point2D, Rect};
use world::{Location, World};
use calx_resource::Resource;
use display;

pub struct View {
    pub world: World,
}

fn on_screen(chart_pos: Point2D<i32>, screen_area: &Rect<f32>) -> Option<i32> {
    let screen_pos = display::chart_to_view(chart_pos) + Point2D::new(320.0, 180.0);
    let bounds = screen_area.inflate(-8.0, -8.0).translate(&Point2D::new(0.0, -4.0));
    if bounds.contains(&screen_pos) {
        Some(-chart_pos.y)
    } else {
        None
    }
}

impl View {
    pub fn new(world: World) -> View { View { world: world } }

    pub fn draw(&mut self, context: &mut display::Context, screen_area: &Rect<f32>) {
        let camera_loc = Location::new(0, 0, 0);

        let center = screen_area.origin + screen_area.size / 2.0;

        // Chart area, center in origin, inflated by tile width in every direction to get the cells
        // partially on screen included.
        let bounds = screen_area.translate(&-(center + screen_area.origin))
                                .inflate(display::PIXEL_UNIT * 2.0, display::PIXEL_UNIT * 2.0);

        let chart = display::screen_fov(&self.world, camera_loc, bounds);

        let mut sprites = Vec::new();

        let cursor_pos = display::view_to_chart(context.ui.mouse_pos() - center);

        for (&chart_pos, origins) in &chart {
            assert!(!origins.is_empty());

            let loc = origins[0] + chart_pos;

            let screen_pos = display::chart_to_view(chart_pos) + center;

            // TODO: Set up dynamic lighting, shade sprites based on angle and local light.
            display::draw_terrain_sprites(&self.world, loc, |layer, _angle, brush, frame_idx| {
                sprites.push(display::Sprite {
                    layer: layer,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: brush.clone(),
                    frame_idx: frame_idx,
                })
            });

            // TODO: Visualization for the on-screen function, remove me.
            if on_screen(chart_pos, screen_area).is_some() {
                sprites.push(display::Sprite {
                    layer: display::Layer::Decal,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: Resource::new("portal".to_string()).unwrap(),
                    frame_idx: 0,
                })
            }
        }

        // Draw cursor.
        if let Some(origins) = chart.get(&cursor_pos) {
            let screen_pos = display::chart_to_view(cursor_pos) + center;
            let loc = origins[0] + cursor_pos;

            // TODO: Need a LOT less verbose API to add stuff to the sprite set.
            sprites.push(display::Sprite {
                layer: display::Layer::Decal,
                offset: [screen_pos.x as i32, screen_pos.y as i32],
                brush: Resource::new("cursor".to_string()).unwrap(),
                frame_idx: 0,
            });
            sprites.push(display::Sprite {
                layer: display::Layer::Effect,
                offset: [screen_pos.x as i32, screen_pos.y as i32],
                brush: Resource::new("cursor_top".to_string()).unwrap(),
                frame_idx: 0,
            });
        }

        sprites.sort();

        for i in &sprites {
            i.draw(&mut context.ui)
        }


        let font = context.ui.default_font();
        context.ui.draw_text(&*font,
                             Point2D::new(0.0, 16.0),
                             [1.0, 1.0, 1.0, 1.0],
                             &format!("Mouse pos {:?}", cursor_pos));
    }
}
