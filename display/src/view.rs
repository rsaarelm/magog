//! Main module for drawing the game map

use crate::{
    cache,
    render::{self, Angle, Layer},
    sprite::{Coloring, Sprite},
    Icon,
};
use calx::{
    project, CellSpace, CellVector, Clamp, FovValue, HexFov, ProjectVec, ProjectVec32, Space,
};
use calx_ecs::Entity;
use euclid::{rect, vec2, vec3, Rect, UnknownUnit, Vector2D, Vector3D};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::Arc;
use vitral::{color, Canvas};
use world::{
    AnimState, FovStatus, LerpLocation, Location, PhysicsSpace, PhysicsVector, Sector, World,
};

/// Useful general constant for cell dimension ops.
pub static PIXEL_UNIT: i32 = 16;

pub struct WorldView {
    pub cursor_loc: Option<Location>,
    pub show_cursor: bool,
    camera_loc: LerpLocation,
    screen_area: ScreenRect,
    fov: Option<HashMap<CellVector, Vec<Location>>>,
}

impl WorldView {
    pub fn new(camera_loc: LerpLocation, screen_area: Rect<i32, UnknownUnit>) -> WorldView {
        WorldView {
            cursor_loc: None,
            show_cursor: false,
            camera_loc,
            screen_area: ScreenRect::from_untyped(&screen_area),
            fov: None,
        }
    }

    /// Convert screen location to cell vector
    pub fn screen_to_cell(&self, pos: ScreenVector) -> Location {
        // NB: This cuts corners with straightforward Location add, if the portaling map system is
        // ever brought back there needs to be a World reference added here for chart lookup.

        // XXX: Repeating the formula in draw
        let center = (self.screen_area.origin + self.screen_area.size / 2
            - vec2(PIXEL_UNIT / 2, 10)
            - self.camera_loc.offset.project())
        .to_vector();
        self.camera_loc.location + (pos - center).project()
    }

    /// Recompute the cached screen view if the cache has been invalidated.
    fn ensure_fov(&mut self, world: &World) {
        if self.fov.is_none() {
            // Chart area, center in origin, inflated by tile width in every direction to get the
            // cells partially on screen included.
            let center = (self.screen_area.origin + self.screen_area.size / 2
                - vec2(PIXEL_UNIT / 2, 0))
            .to_vector();
            let bounds = self
                .screen_area
                .translate(-(self.screen_area.origin + center).to_vector())
                .inflate(PIXEL_UNIT * 2, PIXEL_UNIT * 2);

            self.fov = Some(screen_fov(world, self.camera_loc.location, bounds));
        }
    }

    pub fn draw(&mut self, world: &World, canvas: &mut Canvas) {
        self.camera_loc = clip_camera(world, self.camera_loc);
        self.ensure_fov(world);

        let center = (self.screen_area.origin + self.screen_area.size / 2
            - vec2(PIXEL_UNIT / 2, 10)
            - self.camera_loc.offset.project())
        .to_vector();
        let chart = self.fov.as_ref().unwrap();
        let mut sprites = Vec::new();
        let mouse_pos = ScreenVector::from_untyped(canvas.mouse_pos().to_vector());
        let cursor_pos = (mouse_pos - center).project();

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

            let screen_pos = chart_pos.project() + center;

            let ambient = world.light_level(loc);

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
                            center_pos.project::<PhysicsSpace>().to_3d().normalize()
                        } else {
                            vec3(-(2.0f32.sqrt()) / 2.0, 2.0f32.sqrt() / 2.0, 0.0)
                        };
                        (0.1..=1.0).clamp(-light_dir.dot(normal))
                    };

                    Coloring::Shaded { ambient, diffuse }
                };
                terrain_sprite_buffer.push(
                    Sprite::new(layer, screen_pos, Arc::clone(brush))
                        .idx(frame_idx)
                        .color(color),
                );
            });

            let mut entity_sprite_buffer = Vec::new();

            let mut mobs = Vec::new();
            let mut items = Vec::new();
            let mut fx = Vec::new();

            for e in world.entities_at(loc) {
                if world.is_mob(e) {
                    mobs.push(e);
                } else if world.is_fx(e) {
                    fx.push(e);
                } else {
                    items.push(e);
                }
            }

            // Draw non-mob entities, these are static and shown in map memory
            // FIXME: This should not use live entity data for the remembered objects, since it
            // will then show the object moving around without the player observing it.
            for &i in &items {
                if let Some(desc) = world.ecs().desc.get(i) {
                    let screen_pos = screen_pos + lerp_offset(world, i);
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
                const BLINK_FRAMES: u64 = 5;

                for &i in &mobs {
                    let screen_pos = screen_pos + lerp_offset(world, i);

                    if let Some(desc) = world.ecs().desc.get(i) {
                        let frame_idx = if world.is_bobbing(i) {
                            ((world.get_anim_tick() / 10) % 2) as usize
                        } else {
                            0
                        };

                        let coloring = {
                            if let Some(anim) = world.anim(i) {
                                match anim.state {
                                    AnimState::MobHurt
                                        if world.get_anim_tick() - anim.anim_start
                                            < BLINK_FRAMES =>
                                    {
                                        Coloring::Solid(color::WHITE)
                                    }
                                    AnimState::MobBlocks
                                        if world.get_anim_tick() - anim.anim_start
                                            < BLINK_FRAMES =>
                                    {
                                        Coloring::Solid(color::RED)
                                    }
                                    _ => Coloring::Shaded {
                                        ambient,
                                        diffuse: 1.0,
                                    },
                                }
                            } else {
                                Coloring::Shaded {
                                    ambient,
                                    diffuse: 1.0,
                                }
                            }
                        };

                        entity_sprite_buffer.push(
                            Sprite::new(Layer::Object, screen_pos, cache::entity(desc.icon))
                                .idx(frame_idx)
                                .color(coloring),
                        );
                        draw_health_pips(&mut entity_sprite_buffer, world, i, screen_pos);
                    }
                }

                for &i in &fx {
                    let screen_pos = screen_pos + lerp_offset(world, i);

                    if let Some(anim) = world.anim(i) {
                        // TODO: Speccing anim drawing is way verbose and messy, how to make it
                        // tighter?
                        if anim.state == AnimState::Gib {
                            // XXX: Frame 5 of the gib anim is the splatter on ground, it doesn't
                            // really work visually that well as part of the animation, so just
                            // show the first four splatter frames here. Maybe refactor the frames
                            // into a separate ground decal / corpse object later?
                            const FRAMES: usize = 4;
                            const FRAME_DURATION: usize = 4;

                            let t = (world.get_anim_tick() - anim.anim_start) as usize;

                            let idx = match t {
                                t if t > (FRAMES * FRAME_DURATION - 1) => None,
                                t => Some(t / FRAME_DURATION),
                            };
                            if let Some(idx) = idx {
                                entity_sprite_buffer.push(
                                    Sprite::new(Layer::Effect, screen_pos, cache::misc(Icon::Gib))
                                        .idx(idx)
                                        .color(Coloring::Shaded {
                                            ambient,
                                            diffuse: 1.0,
                                        }),
                                );
                            }
                        }

                        if anim.state == AnimState::Smoke {
                            const FRAMES: usize = 5;
                            const FRAME_DURATION: usize = 4;

                            let t = (world.get_anim_tick() - anim.anim_start) as usize;

                            let idx = match t {
                                t if t > (FRAMES * FRAME_DURATION - 1) => None,
                                t => Some(t / FRAME_DURATION),
                            };
                            if let Some(idx) = idx {
                                entity_sprite_buffer.push(
                                    Sprite::new(
                                        Layer::Effect,
                                        screen_pos,
                                        cache::misc(Icon::Smoke),
                                    )
                                    .idx(idx)
                                    .color(Coloring::Shaded {
                                        ambient,
                                        diffuse: 1.0,
                                    }),
                                );
                            }
                        }

                        if anim.state == AnimState::Explosion {
                            const FRAMES: usize = 8;
                            const FRAME_DURATION: usize = 2;

                            let t = (world.get_anim_tick() - anim.anim_start) as usize;

                            let idx = match t {
                                t if t > (FRAMES * FRAME_DURATION - 1) => None,
                                t => Some(t / FRAME_DURATION),
                            };
                            if let Some(idx) = idx {
                                entity_sprite_buffer.push(
                                    Sprite::new(
                                        Layer::Effect,
                                        screen_pos,
                                        cache::misc(Icon::Explosion),
                                    )
                                    .idx(idx)
                                    .color(Coloring::Shaded {
                                        ambient: 1.0, // Be bright
                                        diffuse: 1.0,
                                    }),
                                );
                            }
                        }

                        if anim.state == AnimState::Firespell {
                            const FRAMES: usize = 2;

                            let t = world.get_anim_tick() - anim.anim_start;

                            if t <= anim.tween_duration as u64 {
                                entity_sprite_buffer.push(
                                    Sprite::new(
                                        Layer::Effect,
                                        screen_pos,
                                        cache::misc(Icon::Firespell),
                                    )
                                    .idx(t as usize % FRAMES)
                                    .color(Coloring::Shaded {
                                        ambient: 1.0, // Be bright
                                        diffuse: 1.0,
                                    }),
                                );
                            }
                        }
                    }
                }
            }

            sprites.extend_from_slice(&terrain_sprite_buffer);
            sprites.extend_from_slice(&entity_sprite_buffer);
        }

        // Draw cursor.
        if let Some(origins) = chart.get(&cursor_pos) {
            let screen_pos = cursor_pos.project() + center;
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
            i.draw(canvas)
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
                let pos = screen_pos + vec2(x * 4 - 10, -10);
                let brush = cache::misc(if x < limit {
                    Icon::HealthPip
                } else {
                    Icon::DarkHealthPip
                });
                sprites.push(Sprite::new(Layer::Effect, pos, brush));
            }
        }

        /// Return vector to add to position if entity's position is being animated.
        fn lerp_offset(world: &World, e: Entity) -> ScreenVector {
            let loc = world.lerp_location(e).unwrap_or_else(Default::default);

            loc.offset.project()
        }
    }
}

#[derive(Clone)]
struct ScreenFov<'a> {
    w: &'a World,
    screen_area: ScreenRect,
    origins: Vec<Location>,
}

impl<'a> PartialEq for ScreenFov<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.w, other.w)
            && self.screen_area == other.screen_area
            && self.origins == other.origins
    }
}

impl<'a> Eq for ScreenFov<'a> {}

impl<'a> FovValue for ScreenFov<'a> {
    fn advance(&self, offset: CellVector) -> Option<Self> {
        if !self.screen_area.contains(offset.project().to_point()) {
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
    screen_area: ScreenRect,
) -> HashMap<CellVector, Vec<Location>> {
    let init = ScreenFov {
        w,
        screen_area,
        origins: vec![origin],
    };

    HashMap::from_iter(HexFov::new(init).map(|(pos, a)| (pos, a.origins)))
}

/// On-screen tile display pixel coordinates.
pub struct ScreenSpace;
impl Space for ScreenSpace {
    type T = i32;
}

// a = PIXEL_UNIT
//
// |   a   -a |
// | a/2  a/2 |

impl project::From<CellSpace> for ScreenSpace {
    fn vec_from(vec: Vector2D<<CellSpace as Space>::T, CellSpace>) -> Vector2D<Self::T, Self> {
        let a = PIXEL_UNIT;
        vec2(vec.x * a - vec.y * a, vec.x * a / 2 + vec.y * a / 2)
    }
}

// a = PIXEL_UNIT
//
// |  1/2a  1/a |
// | -1/2a  1/a |

impl project::From<ScreenSpace> for CellSpace {
    fn vec_from(vec: Vector2D<<ScreenSpace as Space>::T, ScreenSpace>) -> Vector2D<Self::T, Self> {
        let vec = vec2::<f32, ScreenSpace>(vec.x as f32, vec.y as f32);

        // Use a custom function here instead of the matrix inverse, because the naive matrix
        // version projects into an isometric grid instead of the more square on-screen hex cells
        // and feels off when aiming with the mouse.
        let c = PIXEL_UNIT as f32 / 2.0;
        let column = ((vec.x + c) / (c * 2.0)).floor();
        let row = ((vec.y - column * c) / (c * 2.0)).floor();
        vec2((column + row) as i32, row as i32)
    }
}

// |  a   0 |
// |  0  -a |

impl project::From32<PhysicsSpace> for ScreenSpace {
    fn vec_from(
        vec: Vector3D<<PhysicsSpace as Space>::T, PhysicsSpace>,
    ) -> Vector2D<Self::T, Self> {
        let a = PIXEL_UNIT as f32;
        vec2((vec.x * a) as i32, (-vec.y * a - vec.z * a) as i32)
    }
}

impl project::From<ScreenSpace> for PhysicsSpace {
    fn vec_from(vec: Vector2D<<ScreenSpace as Space>::T, ScreenSpace>) -> Vector2D<Self::T, Self> {
        let vec = vec2::<f32, ScreenSpace>(vec.x as f32, vec.y as f32);
        let a = PIXEL_UNIT as f32;
        vec2(vec.x / a, -vec.y / a)
    }
}

pub type ScreenVector = Vector2D<i32, ScreenSpace>;
pub type ScreenRect = Rect<i32, ScreenSpace>;

/// Constrain a scrolling camera when there's no sector to scroll to.
fn clip_camera(world: &World, camera_loc: LerpLocation) -> LerpLocation {
    // XXX: Some messy stuff going on here due to the original CellSpace design not being good
    // with non-integer coordinates.

    let sector = Sector::from(camera_loc.location);

    let center = sector.center();
    // Construct a screen space rectangle where camera position must stay in.
    // Origin is the center of the sector camera_loc is in.
    let screen_bounds: Rect<i32, ScreenSpace> = {
        use world::{SectorDir::*, SectorVec};
        // Start with a rectangle, halfway to neighboring sectors, lots of buffer.
        let p0 = (sector + vec3(-1, -1, 0)).center();
        let p1 = (sector + vec3(1, 1, 0)).center();

        let (mut min_x, mut min_y) = center
            .v2_at(p0)
            .unwrap()
            .project::<ScreenSpace>()
            .to_tuple();
        let (mut max_x, mut max_y) = center
            .v2_at(p1)
            .unwrap()
            .project::<ScreenSpace>()
            .to_tuple();

        // For each neighboring sector group that does not exist, block scrolling towards that
        // direction.
        if ![Northeast, East, Southeast]
            .iter()
            .any(|&d| world.sector_exists(sector + SectorVec::from(d)))
        {
            max_x = 0;
        }
        if ![Northwest, West, Southwest]
            .iter()
            .any(|&d| world.sector_exists(sector + SectorVec::from(d)))
        {
            min_x = 0
        }
        if ![Southeast, Southwest]
            .iter()
            .any(|&d| world.sector_exists(sector + SectorVec::from(d)))
        {
            max_y = 0;
        }
        if ![Northwest, Northeast]
            .iter()
            .any(|&d| world.sector_exists(sector + SectorVec::from(d)))
        {
            min_y = 0
        }

        rect(min_x, min_y, max_x - min_x, max_y - min_y)
    };

    let camera_pos =
        (center.v2_at(camera_loc.location).unwrap()).project() + camera_loc.offset.project();
    let camera_pos = screen_bounds.clamp(camera_pos.to_point());

    let (vec, offset) = screen_space_to_lerp_location(camera_pos.to_vector());
    LerpLocation {
        location: center + vec,
        offset,
    }
}

fn screen_space_to_lerp_location(screen_vector: ScreenVector) -> (CellVector, PhysicsVector) {
    let cell_vector: CellVector = screen_vector.project();

    (
        cell_vector,
        (screen_vector.project::<PhysicsSpace>() - cell_vector.project::<PhysicsSpace>()).to_3d(),
    )
}

#[cfg(test)]
mod test {
    use super::{ScreenSpace, ScreenVector};
    use calx::{CellSpace, CellVector, ProjectVec, ProjectVec32};
    use euclid::vec2;
    use world::PhysicsSpace;

    #[test]
    fn test_projections() {
        for y in -10..10 {
            for x in -10..10 {
                let pos: CellVector = vec2(x, y);
                assert_eq!(
                    pos.project::<ScreenSpace>(),
                    pos.project::<PhysicsSpace>()
                        .to_3d()
                        .project::<ScreenSpace>(),
                );
            }
        }

        for y in -10..10 {
            for x in -10..10 {
                let pos: ScreenVector = vec2(x * 10, y * 10);
                let p1 = pos.project::<PhysicsSpace>().to_3d().project::<CellSpace>();
                let p2 = pos.project::<CellSpace>();
                assert!((p2 - p1).cast::<f32>().length() <= 1.42);
            }
        }
    }
}
