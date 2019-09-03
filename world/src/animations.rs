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
    fn anim_tick(&self) -> u64;
    fn anim(&self, e: Entity) -> Option<&Anim>;
    fn anim_mut(&mut self, e: Entity) -> Option<&mut Anim>;

    /// Advance animations without ticking the world logic.
    ///
    /// Use this when waiting for player input to finish pending animations.
    fn tick_anims(&mut self);

    /// Return whether entity is a transient effect.
    fn is_fx(&self, e: Entity) -> bool {
        if let Some(anim) = self.anim(e) {
            use AnimState::*;
            match anim.state {
                Mob | MobHurt | MobBlocks => false,
                Explosion | Gib => true,
            }
        } else {
            false
        }
    }

    /// Return whether an entity is a transient effect and should be deleted.
    fn is_expired_fx(&self, e: Entity) -> bool {
        if let Some(anim) = self.anim(e) {
            // TODO: Common place to store exact durations of anims, clean the fx immediately when
            // its duration is over.
            // While waiting for that, just clean them up after around 10 seconds
            self.is_fx(e) && self.anim_tick() - anim.anim_start > 300
        } else {
            false
        }
    }

    /// If an entity is undergoing animation, return the current frame
    fn anim_frame(&self, e: Entity) -> Option<usize> {
        unimplemented!();
    }

    /// Return vector by which entity's current position tweening frame displaces it from its base
    /// location.
    ///
    /// Since the current projection system is fixed to integer-coordinate cell vectors, the return
    /// value is scalar a and cell vector v, with the actual displacement vector being a * v.
    fn tween_displacement_vector(&self, e: Entity) -> (f32, CellVector) {
        if let (Some(anim), Some(origin)) = (self.anim(e), self.location(e)) {
            let frame = (self.anim_tick() - anim.tween_start) as u32;
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
    /// An explosion
    Explosion,
    /// A death gib
    Gib,
}

impl Default for AnimState {
    fn default() -> Self { AnimState::Mob }
}
