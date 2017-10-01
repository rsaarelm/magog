use Icon;
use backend::MagogContext;
use brush::Frame;
use cache;
use calx_alg::timing;
use calx_color::Rgba;
use calx_ecs::Entity;
use calx_grid::{FovValue, HexFov};
use euclid::{Point2D, point2, Vector2D, vec2, Rect};
use render::{self, Layer};
use sprite::Sprite;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;
use world::{FovStatus, Location, Query, Sector, TerrainQuery, World};

/// Useful general constant for cell dimension ops.
pub static PIXEL_UNIT: f32 = 16.0;

pub struct WorldView {
    pub cursor_loc: Option<Location>,
    pub show_cursor: bool,
    camera_loc: Location,
    screen_area: Rect<f32>,
    fov: Option<HashMap<Vector2D<i32>, Vec<Location>>>,

    /// Mostly used in mapedit
    pub highlight_offscreen_tiles: bool,
}

impl WorldView {
    pub fn new(camera_loc: Location, screen_area: Rect<f32>) -> WorldView {
        WorldView {
            cursor_loc: None,
            show_cursor: false,
            camera_loc: camera_loc,
            screen_area: screen_area,
            fov: None,
            highlight_offscreen_tiles: false,
        }
    }

    pub fn set_camera(&mut self, loc: Location) {
        if loc != self.camera_loc {
            self.camera_loc = loc;
            self.fov = None;
        }
    }

    /// Recompute the cached screen view if the cache has been invalidated.
    fn ensure_fov(&mut self, world: &World) {
        if self.fov.is_none() {
            // Chart area, center in origin, inflated by tile width in every direction to get the cells
            // partially on screen included.
            let center = (self.screen_area.origin + self.screen_area.size / 2.0 -
                              vec2(PIXEL_UNIT / 2.0, 0.0)).to_vector();
            let bounds = self.screen_area
                .translate(&-(self.screen_area.origin + center).to_vector())
                .inflate(PIXEL_UNIT * 2.0, PIXEL_UNIT * 2.0);

            self.fov = Some(screen_fov(world, self.camera_loc, bounds));
        }
    }

    pub fn draw<C: MagogContext>(&mut self, world: &World, context: &mut C) {
        self.ensure_fov(world);

        let center = (self.screen_area.origin + self.screen_area.size / 2.0 -
                          vec2(PIXEL_UNIT / 2.0, 10.0)).to_vector();
        let chart = self.fov.as_ref().unwrap();
        let mut sprites = Vec::new();
        let cursor_pos = view_to_chart(context.mouse_pos() - center);

        for (&chart_pos, origins) in chart.iter() {
            assert!(!origins.is_empty());

            let mut loc = origins[0] + chart_pos;

            let in_map_memory;

            // If the chart position is in live FOV, we want to show the deepest stack coordinate.
            // If it's not, we want to start looking for the map memory one from the top of the
            // stack.
            if get_fov(world, loc) == Some(FovStatus::Seen) {
                in_map_memory = false;
            } else {
                in_map_memory = true;
                // Only accept map memory from the base layer (top of origins stack)
                loc = origins[origins.len() - 1] + chart_pos;

                let mut gate_point = false;
                // Special case for terrain immediately below a portal when the portal destination
                // is in map memory, it should be drawn as a gate icon in map memory.
                if let Some(endpoint) = world.portal(loc) {
                    if get_fov(world, endpoint) == Some(FovStatus::Remembered) {
                        gate_point = true;
                    }
                }

                if !gate_point && get_fov(world, loc) != Some(FovStatus::Remembered) {
                    // Bail out if there's no memory
                    continue;
                }
            }

            let screen_pos = chart_to_view(chart_pos.to_point()) + center;

            // TODO: Set up dynamic lighting, shade sprites based on angle and local light.
            render::draw_terrain_sprites(world, loc, |layer, _angle, brush, frame_idx| {
                sprites.push(Sprite {
                    layer: layer,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: if in_map_memory {
                        map_memory_colorize(brush.clone())
                    } else {
                        brush.clone()
                    },
                    frame_idx: frame_idx,
                })
            });

            // Draw entities in directly seen cells
            // TODO: Items should be drawn even in map memory
            if !in_map_memory {
                for &i in &world.entities_at(loc) {
                    if let Some(desc) = world.ecs().desc.get(i) {
                        let layer = if world.is_mob(i) {
                            Layer::Object
                        } else {
                            Layer::Items
                        };
                        let frame_idx = if world.is_bobbing(i) {
                            timing::cycle_anim(1.0 / 3.0, 2)
                        } else {
                            0
                        };
                        sprites.push(Sprite {
                            layer: layer,
                            offset: [screen_pos.x as i32, screen_pos.y as i32],
                            brush: cache::entity(desc.icon),
                            frame_idx: frame_idx,
                        });

                        draw_health_pips(&mut sprites, world, i, screen_pos);
                    }
                }
            }

            // XXX: This doesn't belong here, figure out a better place for debug visualizations
            if self.highlight_offscreen_tiles {
                if loc.sector() == Sector::new(0, 0, 0) {
                    sprites.push(Sprite {
                        layer: Layer::Effect,
                        offset: [screen_pos.x as i32, screen_pos.y as i32],
                        brush: cache::misc(Icon::CursorBottom),
                        frame_idx: 0,
                    });
                }
            }
        }

        // Draw cursor.
        if let Some(origins) = chart.get(&cursor_pos.to_vector()) {
            let screen_pos = chart_to_view(cursor_pos) + center;
            let loc = origins[0] + cursor_pos.to_vector();
            self.cursor_loc = Some(loc);

            if self.show_cursor {
                // TODO: Need a LOT less verbose API to add stuff to the sprite set.
                sprites.push(Sprite {
                    layer: Layer::Decal,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: cache::misc(Icon::CursorBottom),
                    frame_idx: 0,
                });
                sprites.push(Sprite {
                    layer: Layer::Effect,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: cache::misc(Icon::CursorTop),
                    frame_idx: 0,
                });
            }
        } else {
            self.cursor_loc = None;
        }

        sprites.sort();

        for i in &sprites {
            i.draw(context)
        }

        fn get_fov(world: &World, loc: Location) -> Option<FovStatus> {
            if let Some(player) = world.player() {
                world.ecs().map_memory.get(player).map_or(
                    Some(FovStatus::Seen),
                    |fov| fov.status(loc),
                )
            } else {
                Some(FovStatus::Seen)
            }
        }

        fn map_memory_colorize(brush: Rc<Vec<Frame>>) -> Rc<Vec<Frame>> {
            // XXX: This is horribly wasteful memory churning.
            use std::ops::Deref;

            let mut ret: Vec<Frame> = brush.deref().clone();
            for frame in ret.iter_mut() {
                for splat in frame.iter_mut() {
                    splat.color = Rgba::from(0x080400ff).into_array();
                    splat.back_color = Rgba::from(0x332200ff).into_array();
                }
            }

            Rc::new(ret)
        }

        fn draw_health_pips(
            sprites: &mut Vec<Sprite>,
            world: &World,
            e: Entity,
            screen_pos: Point2D<f32>,
        ) {
            if !world.is_mob(e) {
                return;
            }

            let hp = world.hp(e);
            let max_hp = world.max_hp(e);

            if hp == max_hp {
                // Perfect health, draw nothing
                return;
            }

            let limit = ((hp * 5) as f32 / max_hp as f32).ceil() as i32;

            for x in 0..5 {
                sprites.push(Sprite {
                    layer: Layer::Effect,
                    offset: [screen_pos.x as i32 + x * 4 - 10, screen_pos.y as i32 - 10],
                    brush: cache::misc(if x < limit {
                        Icon::HealthPip
                    } else {
                        Icon::DarkHealthPip
                    }),
                    frame_idx: 0,
                });
            }
        }
    }
}

/// Transform from chart space (unit is one map cell) to view space (unit is
/// one pixel).
pub fn chart_to_view(chart_pos: Point2D<i32>) -> Point2D<f32> {
    point2(
        (chart_pos.x as f32 * PIXEL_UNIT - chart_pos.y as f32 * PIXEL_UNIT),
        (chart_pos.x as f32 * PIXEL_UNIT / 2.0 + chart_pos.y as f32 * PIXEL_UNIT / 2.0),
    )
}

/// Transform from view space (unit is one pixel) to chart space (unit is one
/// map cell).
pub fn view_to_chart(view_pos: Point2D<f32>) -> Point2D<i32> {
    let c = PIXEL_UNIT / 2.0;
    let column = ((view_pos.x + c) / (c * 2.0)).floor();
    let row = ((view_pos.y - column * c) / (c * 2.0)).floor();
    point2((column + row) as i32, row as i32)
}


#[derive(Clone)]
struct ScreenFov<'a> {
    w: &'a World,
    screen_area: Rect<f32>,
    origins: Vec<Location>,
}

impl<'a> PartialEq for ScreenFov<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.w as *const World == other.w as *const World &&
            self.screen_area == other.screen_area && self.origins == other.origins
    }
}

impl<'a> Eq for ScreenFov<'a> {}

impl<'a> FovValue for ScreenFov<'a> {
    fn advance(&self, offset: Vector2D<i32>) -> Option<Self> {
        if !self.screen_area.contains(&chart_to_view(offset.to_point())) {
            return None;
        }

        let loc = self.origins[0] + offset;

        let mut ret = self.clone();
        // Go through a portal if terrain on our side of the portal is a void cell.
        //
        // With non-void terrain on top of the portal, just show our side and stay on the current
        // frame as far as FOV is concerned.
        if let Some(dest) = self.w.visible_portal(loc) {
            if self.w.is_border_portal(loc) {
                // A border portal will just overwrite the current origin. Map memory will
                // transition seamlessly to show the things past the border portal
                ret.origins[0] = dest - offset;
            } else {
                // A hole portal will add a new layer to the origins stack,
                // we still need to be able to show the previous layer when
                // showing map memory.
                ret.origins.insert(0, dest - offset);
            }
        }

        Some(ret)
    }
}

/// Return the field of view chart for drawing a screen.
///
/// The stack of locations in the return value lists origins for coordinate frames that have been
/// passed through when traversing portals, in reverse order. The first value is the origin of the
/// coordinate space you probably want to show for that point.
pub fn screen_fov(
    w: &World,
    origin: Location,
    screen_area: Rect<f32>,
) -> HashMap<Vector2D<i32>, Vec<Location>> {
    let init = ScreenFov {
        w: w,
        screen_area: screen_area,
        origins: vec![origin],
    };

    HashMap::from_iter(HexFov::new(init).map(|(pos, a)| (pos, a.origins)))
}

#[cfg(test)]
mod test {
    // FIXME: Allow constructing World instances without resource dependencies to allow lightweight
    // unit tests.
/*
    use euclid::{Point2D, Vector2D, Rect, Size2D};
    use world::{Location, Portal, World, Terraform};
    use super::{screen_fov, view_to_chart};

    fn test_world() -> World {
        use world::terrain::{Form, Kind, Tile};
        use calx_resource::ResourceStore;
        use world::Brush;
        use content;

        Brush::insert_resource("dummy".to_string(), Brush::new(Vec::new()));
        Brush::insert_resource("player".to_string(), Brush::new(Vec::new()));

        Tile::insert_resource(0, Tile::new("dummy", Kind::Block, Form::Void));
        Tile::insert_resource(1, Tile::new("dummy", Kind::Ground, Form::Gate));
        Tile::insert_resource(2, Tile::new("dummy", Kind::Ground, Form::Floor));
        Tile::insert_resource(3, Tile::new("dummy", Kind::Ground, Form::Floor));

        let mut ret = World::new(1);

        ret.set_terrain(Location::new(10, 10, 0), 2);
        ret.set_terrain(Location::new(11, 11, 0), 2);
        ret.set_terrain(Location::new(9, 9, 0), 2);
        ret.set_terrain(Location::new(10, 11, 0), 2);
        ret.set_terrain(Location::new(9, 10, 0), 2);
        ret.set_terrain(Location::new(10, 9, 0), 2);

        // Void for the see-through portal.
        ret.set_terrain(Location::new(11, 10, 0), 0);
        ret.set_portal(Location::new(11, 10, 0),
                       Portal::new(Location::new(11, 10, 0), Location::new(31, 10, 0)));
        ret.set_terrain(Location::new(31, 10, 0), 3);

        ret
    }

    #[test]
    fn test_portaling_fov() {
        let world = test_world();
        let fov = screen_fov(&world,
                             Location::new(10, 10, 0),
                             Rect::new(Point2D::new(-48.0, -48.0), Size2D::new(96.0, 96.0)));
        assert_eq!(fov.get(&vec2(0, 0)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&vec2(0, 1)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&vec2(1, 1)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&vec2(-1, -1)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&vec2(1, 0)),
                   Some(&vec![Location::new(30, 10, 0), Location::new(10, 10, 0)]));
    }

    #[test]
    fn test_corner_visibility() {
        let world = test_world();
        let screen_rect = Rect::new(Point2D::new(-200.0, -200.0), Size2D::new(400.0, 400.0));
        let fov = screen_fov(&world, Location::new(10, 10, 0), screen_rect);

        // Check that the fov is bounded close to the given rectangle.

        let inside_screen = screen_rect.inflate(-40.0, -40.0);
        assert!(fov.get(&view_to_chart(inside_screen.origin)).is_some());
        assert!(fov.get(&view_to_chart(inside_screen.bottom_left())).is_some());
        assert!(fov.get(&view_to_chart(inside_screen.top_right())).is_some());
        assert!(fov.get(&view_to_chart(inside_screen.bottom_right())).is_some());

        let outside_screen = screen_rect.inflate(40.0, 40.0);
        assert!(fov.get(&view_to_chart(outside_screen.origin)).is_none());
        assert!(fov.get(&view_to_chart(outside_screen.bottom_left())).is_none());
        assert!(fov.get(&view_to_chart(outside_screen.top_right())).is_none());
        assert!(fov.get(&view_to_chart(outside_screen.bottom_right())).is_none());
    }
*/


}
