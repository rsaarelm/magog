//! Animation effects embedded in the game world.

use crate::{location::Location, world::World};
use calx::{ease, project, CellSpace, Space};
use calx_ecs::Entity;
use euclid::{vec2, Vector2D, Vector3D};
use serde_derive::{Deserialize, Serialize};

/// Location with a non-integer offset delta.
///
/// Use for tweened animations.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct LerpLocation {
    location: Location,
    offset: Vector2D<f32, CellSpace>,
}

impl LerpLocation {
    pub fn new(location: Location, offset: Vector2D<f32, CellSpace>) -> LerpLocation {
        // XXX: This could be normalized so that location gets displaced if offset is longer than a
        // cell's width, but that doesn't work with the current display code that always draws
        // entities in their logical location, not whatever LerpLocation says it is. Would need to
        // develop a new spatial index that finds entities in the cell of LerpLocation for drawing.
        // (This is mostly relevant to projectiles that fly across several cells).
        //
        // As it stands, the location field is a bit useless since it will always match the
        // entity's logical location. It used to get moved before I noticed the bug with it.
        LerpLocation {
            location,
            offset: vec2(offset.x, offset.y),
        }
    }

    #[inline]
    pub fn location(self) -> Location { self.location }

    #[inline]
    pub fn offset(self) -> Vector2D<f32, CellSpace> { self.offset }
}

impl From<Location> for LerpLocation {
    fn from(location: Location) -> LerpLocation {
        LerpLocation {
            location,
            offset: Default::default(),
        }
    }
}

impl World {
    pub fn get_anim_tick(&self) -> u64 { self.flags.anim_tick }
    pub fn anim(&self, e: Entity) -> Option<&Anim> { self.ecs.anim.get(e) }
    pub(crate) fn anim_mut(&mut self, e: Entity) -> Option<&mut Anim> { self.ecs.anim.get_mut(e) }
    /// Advance animations without ticking the world logic.
    ///
    /// Use this when waiting for player input to finish pending animations.
    pub fn tick_anims(&mut self) { self.flags.anim_tick += 1; }

    /// Return whether entity is a transient effect.
    pub fn is_fx(&self, e: Entity) -> bool {
        self.anim(e)
            .map_or(false, |a| a.state.is_transient_anim_state())
    }

    /// Return a location structure that includes the entity's animation displacement
    pub fn lerp_location(&self, e: Entity) -> Option<LerpLocation> {
        if let Some(location) = self.location(e) {
            if let Some(anim) = self.anim(e) {
                let frame = (self.get_anim_tick() - anim.tween_start) as u32;
                if frame < anim.tween_duration {
                    if let Some(vec) = location.v2_at(anim.tween_from) {
                        let scalar = frame as f32 / anim.tween_duration as f32;
                        let scalar = ease::cubic_in_out(1.0 - scalar);

                        let offset: euclid::Vector2D<f32, CellSpace> = vec.cast();
                        return Some(LerpLocation::new(location, offset * scalar));
                    }
                }
            }
            Some(location.into())
        } else {
            None
        }
    }
}

/// Entity animation state.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Anim {
    pub tween_from: Location,
    /// Anim_tick when tweening started
    pub tween_start: u64,
    /// How many frames does the tweening take
    pub tween_duration: u32,
    /// Anim_tick when the animation started
    pub anim_start: u64,
    /// World tick when anim should be cleaned up.
    ///
    /// NB: Both entity creation and destruction must use world logic and world clock, not the
    /// undeterministic animation clock. Deleting entities at unspecified time points through
    /// animation logic can inject indeterminism in world progress.
    pub anim_done_world_tick: Option<u64>,
    pub state: AnimState,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum AnimState {
    /// Mob decorator, doing nothing in particular
    Mob,
    /// Show mob hurt animation
    MobHurt,
    /// Show mob blocking autoexplore animation
    MobBlocks,
    /// A death gib
    Gib,
    /// Puff of smoke
    Smoke,
    /// Single-cell explosion
    Explosion,
    /// Pre-exploded fireball
    Firespell,
}

impl AnimState {
    /// This is a state that belongs to an animation that gets removed, not a status marker for a
    /// permanent entity.
    pub fn is_transient_anim_state(self) -> bool {
        use AnimState::*;
        match self {
            Mob | MobHurt | MobBlocks => false,
            Gib | Smoke | Explosion | Firespell => true,
        }
    }
}

impl Default for AnimState {
    fn default() -> Self { AnimState::Mob }
}

/// 3D physics space, used for eg. lighting.
pub struct PhysicsSpace;
impl Space for PhysicsSpace {
    type T = f32;
}

// |    1    -1 |
// | -1/2  -1/2 |

impl project::From<CellSpace> for PhysicsSpace {
    fn vec_from(vec: Vector2D<<CellSpace as Space>::T, CellSpace>) -> Vector2D<Self::T, Self> {
        let vec = vec.cast::<f32>();
        vec2(vec.x - vec.y, -vec.x / 2.0 - vec.y / 2.0)
    }
}

// |  1/2  -1 |
// | -1/2  -1 |

impl project::From<PhysicsSpace> for CellSpace {
    fn vec_from(
        vec: Vector2D<<PhysicsSpace as Space>::T, PhysicsSpace>,
    ) -> Vector2D<Self::T, Self> {
        vec2(
            (vec.x / 2.0 - vec.y).round() as i32,
            (-vec.x / 2.0 - vec.y).round() as i32,
        )
    }
}

pub type PhysicsVector = Vector3D<f32, PhysicsSpace>;
