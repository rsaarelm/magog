use std::collections::HashMap;
use std::iter::FromIterator;
use euclid::{Point2D, Rect};
use calx_grid::{Dir6, FovValue, HexFov};
use world::{self, FovStatus, Location, Query, TerrainQuery, World};
use sprite::Sprite;
use render::{self, Layer};
use backend;
use cache;
use Icon;

/// Useful general constant for cell dimension ops.
pub static PIXEL_UNIT: f32 = 16.0;

pub struct WorldView {
    pub cursor_loc: Option<Location>,
    pub show_cursor: bool,
    camera_loc: Location,
    screen_area: Rect<f32>,
    fov: Option<HashMap<Point2D<i32>, Vec<Location>>>,

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
            let center = self.screen_area.origin + self.screen_area.size / 2.0 -
                         Point2D::new(PIXEL_UNIT / 2.0, 0.0);
            let bounds = self.screen_area
                             .translate(&-(center + self.screen_area.origin))
                             .inflate(PIXEL_UNIT * 2.0, PIXEL_UNIT * 2.0);

            self.fov = Some(screen_fov(world, self.camera_loc, bounds));
        }
    }

    pub fn draw(&mut self, world: &World, context: &mut backend::Context) {
        self.ensure_fov(world);

        let center = self.screen_area.origin + self.screen_area.size / 2.0 -
                     Point2D::new(PIXEL_UNIT / 2.0, 0.0);
        let chart = self.fov.as_ref().unwrap();
        let mut sprites = Vec::new();
        let cursor_pos = view_to_chart(context.ui.mouse_pos() - center);

        let mut fov_status = Some(FovStatus::Seen);

        for (&chart_pos, origins) in chart.iter() {
            assert!(!origins.is_empty());

            let loc = origins[0] + chart_pos;

            // Always draw FOV if there's an active player with a map memory component.
            if let Some(player) = world.player() {
                fov_status = world.ecs().map_memory.get(player).map_or(None, |fov| fov.status(loc));
            }

            if fov_status.is_none() {
                continue;
            }

            if fov_status == Some(FovStatus::Remembered) {
                // TODO: Map memory display.
                continue;
            }

            let screen_pos = chart_to_view(chart_pos) + center;

            // TODO: Set up dynamic lighting, shade sprites based on angle and local light.
            render::draw_terrain_sprites(&world, loc, |layer, _angle, brush, frame_idx| {
                sprites.push(Sprite {
                    layer: layer,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: brush.clone(),
                    frame_idx: frame_idx,
                })
            });

            for &i in &world.entities_at(loc) {
                if let Some(desc) = world.ecs().desc.get(i) {
                    let layer = if world.is_mob(i) { Layer::Object } else { Layer::Items };
                    let frame_idx = if world.is_bobbing(i) { 1 } else { 0 };
                    sprites.push(Sprite {
                        layer: layer,
                        offset: [screen_pos.x as i32, screen_pos.y as i32],
                        brush: cache::entity(desc.icon),
                        frame_idx: frame_idx,
                    });
                }
            }

            if self.highlight_offscreen_tiles {
                if let Some(loc) = Location::origin().v2_at(loc) {
                    if !world::on_screen(loc) {
                        if Dir6::iter().any(|d| world::on_screen(loc + d.to_v2())) {
                            sprites.push(Sprite {
                                layer: Layer::Effect,
                                offset: [screen_pos.x as i32, screen_pos.y as i32],
                                brush: cache::misc(Icon::Portal),
                                frame_idx: 0,
                            });
                        } else {
                            sprites.push(Sprite {
                                layer: Layer::Effect,
                                offset: [screen_pos.x as i32, screen_pos.y as i32],
                                brush: cache::misc(Icon::CursorBottom),
                                frame_idx: 0,
                            });
                        }
                    }
                }
            }
        }

        // Draw cursor.
        if let Some(origins) = chart.get(&cursor_pos) {
            let screen_pos = chart_to_view(cursor_pos) + center;
            let loc = origins[0] + cursor_pos;
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
            i.draw(&mut context.ui)
        }
    }
}

/// Transform from chart space (unit is one map cell) to view space (unit is
/// one pixel).
pub fn chart_to_view(chart_pos: Point2D<i32>) -> Point2D<f32> {
    Point2D::new((chart_pos.x as f32 * PIXEL_UNIT - chart_pos.y as f32 * PIXEL_UNIT),
                 (chart_pos.x as f32 * PIXEL_UNIT / 2.0 + chart_pos.y as f32 * PIXEL_UNIT / 2.0))
}

/// Transform from view space (unit is one pixel) to chart space (unit is one
/// map cell).
pub fn view_to_chart(view_pos: Point2D<f32>) -> Point2D<i32> {
    let c = PIXEL_UNIT / 2.0;
    let column = ((view_pos.x + c) / (c * 2.0)).floor();
    let row = ((view_pos.y - column * c) / (c * 2.0)).floor();
    Point2D::new((column + row) as i32, row as i32)
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
    fn advance(&self, offset: Point2D<i32>) -> Option<Self> {
        if !self.screen_area.contains(&chart_to_view(offset)) {
            return None;
        }

        let loc = self.origins[0] + offset;

        let mut ret = self.clone();
        // Go through a portal if terrain on our side of the portal is a void cell.
        //
        // With non-void terrain on top of the portal, just show our side and stay on the current
        // frame as far as FOV is concerned.
        if let Some(dest) = self.w.visible_portal(loc) {
            ret.origins.insert(0, dest - offset);
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
    screen_area: Rect<f32>
) -> HashMap<Point2D<i32>, Vec<Location>> {
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
    use euclid::{Point2D, Rect, Size2D};
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
        assert_eq!(fov.get(&Point2D::new(0, 0)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&Point2D::new(0, 1)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&Point2D::new(1, 1)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&Point2D::new(-1, -1)),
                   Some(&vec![Location::new(10, 10, 0)]));

        assert_eq!(fov.get(&Point2D::new(1, 0)),
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
