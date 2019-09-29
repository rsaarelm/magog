//! Animation effects embedded in the game world.

use crate::location::Location;
use crate::query::Query;
use calx::{ease, CellVector};
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};

/// Trait for advancing animations.
///
/// Kept separate from trait `Mutate` to emphasize the contract that advancing animation logic must
/// never cause game logic changes.
pub trait Animations: Query + Sized {
    fn get_anim_tick(&self) -> u64;
    fn anim(&self, e: Entity) -> Option<&Anim>;
    fn anim_mut(&mut self, e: Entity) -> Option<&mut Anim>;

    /// Advance animations without ticking the world logic.
    ///
    /// Use this when waiting for player input to finish pending animations.
    fn tick_anims(&mut self);

    /// Return whether entity is a transient effect.
    fn is_fx(&self, e: Entity) -> bool {
        self.anim(e)
            .map_or(false, |a| a.state.is_transient_anim_state())
    }

    /// Return vector by which entity's current position tweening frame displaces it from its base
    /// location.
    ///
    /// Since the current projection system is fixed to integer-coordinate cell vectors, the return
    /// value is scalar a and cell vector v, with the actual displacement vector being a * v.
    fn tween_displacement_vector(&self, e: Entity) -> (f32, CellVector) {
        if let (Some(anim), Some(origin)) = (self.anim(e), self.location(e)) {
            let frame = (self.get_anim_tick() - anim.tween_start) as u32;
            if frame < anim.tween_duration {
                if let Some(vec) = origin.v2_at(anim.tween_from) {
                    let scalar = frame as f32 / anim.tween_duration as f32;
                    return (ease::cubic_in_out(1.0 - scalar), vec);
                }
            }
        }
        (0.0, Default::default())
    }
}

/// Entity animation state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
