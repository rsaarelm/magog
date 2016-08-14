use euclid::Rect;
use world::{Location, World};
use sprite::Sprite;
use backend;
use view;
use render;

/// Top-level application state for gameplay.
pub struct GameView {
    pub world: World,
}

impl GameView {
    pub fn new(world: World) -> GameView {
        GameView { world: world }
    }

    pub fn draw(&self, context: &mut backend::Context, screen_area: &Rect<f32>) {
        // TODO: Camera logic
        let camera_loc = Location::new(0, 0);

        let center = screen_area.origin + screen_area.size / 2.0;

        // Chart area, center in origin, inflated by tile width in every direction to get the cells
        // partially on screen included.
        let bounds = screen_area.translate(&-(center + screen_area.origin))
                                .inflate(view::PIXEL_UNIT * 2.0, view::PIXEL_UNIT * 2.0);

        context.set_clip_rect(Some(*screen_area));

        let chart = view::screen_fov(&self.world, camera_loc, bounds);

        let mut sprites = Vec::new();

        for (&chart_pos, origins) in chart.iter() {
            assert!(!origins.is_empty());

            let loc = origins[0] + chart_pos;

            let screen_pos = view::chart_to_view(chart_pos) + center;

            // TODO: Set up dynamic lighting, shade sprites based on angle and local light.
            render::draw_terrain_sprites(&self.world, loc, |layer, _angle, brush, frame_idx| {
                sprites.push(Sprite {
                    layer: layer,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: brush.clone(),
                    frame_idx: frame_idx,
                })
            });
        }

        sprites.sort();

        for i in sprites.iter() {
            i.draw(context)
        }

        context.set_clip_rect(None);
    }
}
