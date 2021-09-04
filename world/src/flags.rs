use crate::location::Location;
use calx_ecs::Entity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Flags {
    pub camera: Location,
    pub tick: u64,
    pub anim_tick: u64,
    pub player_acted: bool,
    /// Store the player entity here for fast access.
    pub player: Option<Entity>,
    pub depth: i32,
}
