use Icon;
use backend::Core;
use cache;
use calx::{clamp, cycle_anim, CellVector, FovValue, HexFov, Space, Transformation};
use calx_ecs::Entity;
use euclid::{Rect, TypedRect, TypedVector2D, TypedVector3D, vec2, vec3};
use render::{self, Angle, Layer};
use sprite::{Coloring, Sprite};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;
use world::{FovStatus, Location, Query, TerrainQuery, World};

/// Useful general constant for cell dimension ops.
pub static PIXEL_UNIT: f32 = 16.0;

pub struct WorldView {
    pub cursor_loc: Option<Location>,
    pub show_cursor: bool,
    camera_loc: Location,
    screen_area: TypedRect<f32, ScreenSpace>,
    fov: Option<HashMap<CellVector, Vec<Location>>>,
}

impl WorldView {
    pub fn new(camera_loc: Location, screen_area: Rect<f32>) -> WorldView {
        WorldView {
            cursor_loc: None,
            show_cursor: false,
            camera_loc: camera_loc,
            screen_area: ScreenRect::from_untyped(&screen_area),
            fov: None,
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
            // Chart area, center in origin, inflated by tile width in every direction to get the
            // cells partially on screen included.
            let center = (self.screen_area.origin + self.screen_area.size / 2.0
                - vec2(PIXEL_UNIT / 2.0, 0.0))
                .to_vector();
            let bounds = self.screen_area
                .translate(&-(self.screen_area.origin + center).to_vector())
                .inflate(PIXEL_UNIT * 2.0, PIXEL_UNIT * 2.0);

            self.fov = Some(screen_fov(world, self.camera_loc, bounds));
        }
    }

    pub fn draw(&mut self, world: &World, core: &mut Core) {
        let current_sector = self.camera_loc.sector();
        self.camera_loc = current_sector.center();

        self.ensure_fov(world);

        let center = (self.screen_area.origin + self.screen_area.size / 2.0
            - vec2(PIXEL_UNIT / 2.0, 10.0))
            .to_vector();
        let chart = self.fov.as_ref().unwrap();
        let mut sprites = Vec::new();
        let mouse_pos = ScreenVector::from_untyped(&core.mouse_pos().to_vector());
        let cursor_pos = (mouse_pos - center).to_cell_space();

        for (&chart_pos, origins) in chart.iter() {
            assert!(!origins.is_empty());

            let mut loc = origins[0] + chart_pos;

            // Relative player pos to currently drawn thing.
            let mut player_pos = None;
            if let Some(player) = world.player() {
                if let Some(player_loc) = world.location(player) {
                    if let Some(vec) = player_loc.v2_at(loc) {
                        player_pos = Some(vec);
                    }
                }
            }

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

            let screen_pos = ScreenVector::from_cell_space(chart_pos) + center;

            let ambient = world.light_level(loc);

            // Tile is outside current sector and can't be entered, graphical cues to point this
            // out may be needed.
            let blocked_offsector =
                loc.sector() != current_sector && world.terrain(loc).is_narrow_obstacle();

            if blocked_offsector && !in_map_memory {
                sprites.push(Sprite::new(
                    Layer::Decal,
                    screen_pos,
                    cache::misc(Icon::BlockedOffSectorCell),
                ));
            }

            let mut terrain_sprite_buffer = Vec::new();

            render::draw_terrain_sprites(world, loc, |layer, angle, brush, frame_idx| {
                let color = if in_map_memory {
                    Coloring::MapMemory
                } else {
                    let diffuse = if angle == Angle::Up || angle == Angle::South {
                        // Angle::South is for all the non-wall props, don't shade them
                        1.0
                    } else {
                        let normal = angle.normal();
                        // When underground, use the player position as light position instead of
                        // constant-dir sunlight.
                        let light_dir = if world.is_underground(loc) && player_pos.is_some() {
                            // XXX: Things get screwy if location is the same as player location
                            // (eg. when player stands in a doorway), hack around that by
                            // displacing the zero position.
                            let center_pos = if player_pos == Some(vec2(0, 0)) {
                                vec2(1, 1)
                            } else {
                                player_pos.unwrap()
                            };
                            PhysicsVector::from_cell_space(center_pos).normalize()
                        } else {
                            vec3(-(2.0f32.sqrt()) / 2.0, 2.0f32.sqrt() / 2.0, 0.0)
                        };
                        clamp(0.1, 1.0, -light_dir.dot(normal))
                    };

                    Coloring::Shaded { ambient, diffuse }
                };
                terrain_sprite_buffer.push(
                    Sprite::new(layer, screen_pos, Rc::clone(brush))
                        .idx(frame_idx)
                        .color(color),
                );
            });

            let mut entity_sprite_buffer = Vec::new();

            let (mobs, items): (Vec<Entity>, Vec<Entity>) = world
                .entities_at(loc)
                .into_iter()
                .partition(|&e| world.is_mob(e));

            // Draw non-mob entities, these are static and shown in map memory
            // FIXME: This should not use live entity data for the remembered objects, since it
            // will then show the object moving around without the player observing it.
            for &i in &items {
                if let Some(desc) = world.ecs().desc.get(i) {
                    let color = if in_map_memory {
                        Coloring::MapMemory
                    } else {
                        Coloring::Shaded {
                            ambient,
                            diffuse: 1.0,
                        }
                    };
                    entity_sprite_buffer.push(
                        Sprite::new(Layer::Object, screen_pos, cache::entity(desc.icon))
                            .color(color),
                    );
                }
            }

            // Draw mobs in directly seen cells
            if !in_map_memory {
                for &i in &mobs {
                    if let Some(desc) = world.ecs().desc.get(i) {
                        let frame_idx = if world.is_bobbing(i) {
                            cycle_anim(1.0 / 3.0, 2)
                        } else {
                            0
                        };
                        entity_sprite_buffer.push(
                            Sprite::new(Layer::Object, screen_pos, cache::entity(desc.icon))
                                .idx(frame_idx)
                                .color(Coloring::Shaded {
                                    ambient,
                                    diffuse: 1.0,
                                }),
                        );
                        draw_health_pips(&mut entity_sprite_buffer, world, i, screen_pos);
                    }
                }
            }

            // A doorway wall should be drawn on top of entities, but regular terrain blocks should
            // go below them.
            //
            // (Disabled. This is visually correct, but the wall graphic ends up obstructing the
            // mob sprite almost completely, and it's more important to be able to see what mob is
            // standing in the doorway than for things to be visually nice.)

            // if world.terrain(loc).is_wall() {
            //     sprites.extend_from_slice(&entity_sprite_buffer);
            //     sprites.extend_from_slice(&terrain_sprite_buffer);
            // } else {
            sprites.extend_from_slice(&terrain_sprite_buffer);
            sprites.extend_from_slice(&entity_sprite_buffer);
            // }
        }

        // Draw cursor.
        if let Some(origins) = chart.get(&cursor_pos) {
            let screen_pos = ScreenVector::from_cell_space(cursor_pos) + center;
            let loc = origins[0] + cursor_pos;
            self.cursor_loc = Some(loc);

            if self.show_cursor {
                sprites.push(Sprite::new(
                    Layer::Decal,
                    screen_pos,
                    cache::misc(Icon::CursorBottom),
                ));
                sprites.push(Sprite::new(
                    Layer::Effect,
                    screen_pos,
                    cache::misc(Icon::CursorTop),
                ));
            }
        } else {
            self.cursor_loc = None;
        }

        sprites.sort();

        for i in &sprites {
            i.draw(core)
        }

        fn get_fov(world: &World, loc: Location) -> Option<FovStatus> {
            if let Some(player) = world.player() {
                world
                    .ecs()
                    .map_memory
                    .get(player)
                    .map_or(Some(FovStatus::Seen), |fov| fov.status(loc))
            } else {
                Some(FovStatus::Seen)
            }
        }

        fn draw_health_pips(
            sprites: &mut Vec<Sprite>,
            world: &World,
            e: Entity,
            screen_pos: ScreenVector,
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
                let pos = screen_pos + vec2(x as f32 * 4.0 - 10.0, -10.0);
                let brush = cache::misc(if x < limit {
                    Icon::HealthPip
                } else {
                    Icon::DarkHealthPip
                });
                sprites.push(Sprite::new(Layer::Effect, pos, brush));
            }
        }
    }
}

#[derive(Clone)]
struct ScreenFov<'a> {
    w: &'a World,
    screen_area: TypedRect<f32, ScreenSpace>,
    origins: Vec<Location>,
}

impl<'a> PartialEq for ScreenFov<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.w as *const World == other.w as *const World && self.screen_area == other.screen_area
            && self.origins == other.origins
    }
}

impl<'a> Eq for ScreenFov<'a> {}

impl<'a> FovValue for ScreenFov<'a> {
    fn advance(&self, offset: CellVector) -> Option<Self> {
        if !self.screen_area
            .contains(&ScreenVector::from_cell_space(offset).to_point())
        {
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
    screen_area: TypedRect<f32, ScreenSpace>,
) -> HashMap<CellVector, Vec<Location>> {
    let init = ScreenFov {
        w: w,
        screen_area: screen_area,
        origins: vec![origin],
    };

    HashMap::from_iter(HexFov::new(init).map(|(pos, a)| (pos, a.origins)))
}

/// On-screen tile display pixel coordinates.
pub struct ScreenSpace;

// a = PIXEL_UNIT
//
// |   a   -a |
// | a/2  a/2 |
//
// |  1/2a  1/a |
// | -1/2a  1/a |

impl Transformation for ScreenSpace {
    type Element = f32;

    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2] {
        let v = v.into();
        let v = [v[0] as f32, v[1] as f32];
        [
            v[0] * PIXEL_UNIT - v[1] * PIXEL_UNIT,
            v[0] * PIXEL_UNIT / 2.0 + v[1] * PIXEL_UNIT / 2.0,
        ]
    }

    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2] {
        let v = v.into();

        // Use a custom function here instead of the matrix inverse, because the naive matrix
        // version projects into an isometric grid instead of the more square on-screen hex cells
        // and feels off when aiming with the mouse.
        let c = PIXEL_UNIT / 2.0;
        let column = ((v[0] + c) / (c * 2.0)).floor();
        let row = ((v[1] - column * c) / (c * 2.0)).floor();
        [(column + row) as i32, row as i32]
    }
}

pub type ScreenVector = TypedVector2D<f32, ScreenSpace>;
pub type ScreenRect = TypedRect<f32, ScreenSpace>;

/// 3D physics space, used for eg. lighting.
pub struct PhysicsSpace;

// |    1    -1 |
// | -1/2  -1/2 |
//
// |  1/2  -1 |
// | -1/2  -1 |

impl Transformation for PhysicsSpace {
    type Element = f32;

    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2] {
        let v = v.into();
        let v = [v[0] as f32, v[1] as f32];
        [v[0] - v[1], -v[0] / 2.0 - v[1] / 2.0]
    }

    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2] {
        let v = v.into();
        [
            (v[0] / 2.0 - v[1]).round() as i32,
            (-v[0] / 2.0 - v[1]).round() as i32,
        ]
    }
}

pub type PhysicsVector = TypedVector3D<f32, PhysicsSpace>;
