//! Creature AI and activity loop logic

use crate::{
    msg,
    stats::{Intrinsic, Status},
    Location, World,
};
use calx::{Dir6, RngExt};
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};

/// Used to determine who tries to fight whom.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Alignment {
    /// Standard dungeon enemies, work in concert against the player.
    Enemy,
    /// Player character and allies, opposed to enemy.
    Player,
    /// Indifferent or hungry, potential threat to Player and Enemy alike
    Animal,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Brain {
    pub state: BrainState,
    pub alignment: Alignment,
    pub shout: ShoutType,
}

impl Brain {
    /// Create default enemy brain.
    pub fn enemy() -> Brain {
        Brain {
            ..Default::default()
        }
    }

    /// Create default player brain.
    pub fn player() -> Brain {
        Brain {
            state: BrainState::PlayerControl,
            alignment: Alignment::Player,
            ..Default::default()
        }
    }

    pub fn shout(mut self, shout: ShoutType) -> Brain {
        self.shout = shout;
        self
    }
}

impl Default for Brain {
    fn default() -> Brain {
        Brain {
            state: BrainState::Asleep,
            alignment: Alignment::Enemy,
            shout: ShoutType::Silent,
        }
    }
}

/// Mob behavior state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum BrainState {
    /// AI mob is inactive, but can be startled into action by noise or
    /// motion.
    Asleep,
    /// AI mob is looking for a fight.
    Hunting(Entity),
    /// Mob is wandering aimlessly
    Roaming,
    /// Mob is under player control.
    PlayerControl,
}

/// How does a mob vocalize when alerted?
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum ShoutType {
    /// Humanoids
    Shout,
    /// Reptiles
    Hiss,
    /// Insects
    Buzz,
    /// Large monsters
    Roar,
    /// Slimes
    Gurgle,
    Bark,
    Meow,
    Squeak,
    Silent,
}

impl World {
    /// Run AI for all autonomous mobs.
    pub(crate) fn ai_main(&mut self) {
        for npc in self.active_mobs() {
            self.heartbeat(npc);

            if !self.is_npc(npc) {
                continue;
            }
            if self.ticks_this_frame(npc) {
                self.run_ai_for(npc)
            }
        }
    }

    /// Run AI for one non-player-controlled creature.
    fn run_ai_for(&mut self, npc: Entity) {
        use BrainState::*;

        // Prevent hyperactive mobs from ever being in asleep state.
        if self.has_intrinsic(npc, Intrinsic::Hyperactive) {
            if let Some(brain) = self.ecs_mut().brain.get_mut(npc) {
                if brain.state == BrainState::Asleep {
                    brain.state = BrainState::Roaming;
                }
            }
        }

        let brain_state = self.brain_state(npc).expect("Running AI for non-mob");
        match brain_state {
            Asleep => {
                // TODO: Noise waves that wake up sleeping mobs when they hit them, stop making
                // each sleeping mob do active scanning every tic.
                if let Some(e) = self.find_enemy(npc) {
                    self.designate_enemy(npc, e);
                }
            }
            Hunting(target) => {
                if self.rng().one_chance_in(12) {
                    self.ai_drift(npc);
                } else {
                    self.ai_hunt(npc, target);
                }
            }
            Roaming => {
                if let Some(e) = self.find_enemy(npc) {
                    self.designate_enemy(npc, e);
                    self.ai_hunt(npc, e);
                } else {
                    self.ai_drift(npc);
                }
            }
            PlayerControl => {}
        }
    }

    /// Approach and attack target entity.
    fn ai_hunt(&mut self, npc: Entity, target: Entity) {
        if let (Some(my_loc), Some(target_loc)) = (self.location(npc), self.location(target)) {
            if my_loc.metric_distance(target_loc) == 1 {
                let _ = self.entity_melee(npc, my_loc.dir6_towards(target_loc).unwrap());
            } else if let Some(move_dir) = self.pathing_dir_towards(npc, target_loc) {
                let _ = self.entity_step(npc, move_dir);
            } else {
                self.ai_drift(npc);
            }
        }
    }

    /// Wander around aimlessly
    fn ai_drift(&mut self, npc: Entity) {
        let dirs = Dir6::permuted_dirs(self.rng());
        for &dir in &dirs {
            if self.entity_step(npc, dir).is_some() {
                return;
            }
        }
    }

    /// Find for an enemy for AI to target
    fn find_enemy(&self, npc: Entity) -> Option<Entity> {
        static FLEE_THRESHOLD: i32 = 14;
        const WAKEUP_DISTANCE: i32 = 5;

        let brain_state = self.brain_state(npc)?;
        if let BrainState::Hunting(x) = brain_state {
            // Is the existing target still valid?
            if self.is_alive(x)
                && self
                    .distance_between(npc, x)
                    .map_or(false, |d| d <= FLEE_THRESHOLD)
            {
                return Some(x);
            }
        }

        // TODO: Possibility to pick other targets than player
        // TODO: Creatures that are friendly or neutral to player

        if let (Some(loc), Some(player), Some(player_loc)) = (
            self.location(npc),
            self.player(),
            self.player().map(|p| self.location(p)).unwrap_or(None),
        ) {
            if self.player_sees(loc) {
                // Okay, tricky spot. Player might be seeing mob across a portal, in
                // which case we can't do naive distance check.
                // This could have a helper method that finds chart distance to self in
                // player's map memory.
                //
                // For now, let's just go with mobs past portals not waking up to
                // player.

                if loc.metric_distance(player_loc) <= WAKEUP_DISTANCE {
                    return Some(player);
                }
            }
        }
        None
    }

    /// End move for entity.
    ///
    /// Applies delay.
    pub(crate) fn end_turn(&mut self, e: Entity) {
        let delay = self.action_delay(e);
        self.gain_status(e, Status::Delayed, delay);
    }

    pub(crate) fn notify_attacked_by(&mut self, victim: Entity, attacker: Entity) {
        // TODO: Check if victim is already in close combat and don't disengage against new target
        // if it is.
        // TODO: The idea is to do Doom-style thing where friendly fire will occasionally cause
        // infighting between enemies, but it should be rare.
        self.designate_enemy(victim, attacker);
    }

    fn designate_enemy(&mut self, e: Entity, target: Entity) {
        // TODO: Probably want this logic to be more complex eventually.
        if self.is_npc(e) {
            if self.brain_state(e) == Some(BrainState::Asleep) {
                self.shout(e);
            }
            if let Some(brain) = self.ecs_mut().brain.get_mut(e) {
                brain.state = BrainState::Hunting(target);
            }
        }
    }

    /// Make a mob shout according to its type.
    pub(crate) fn shout(&mut self, e: Entity) {
        // TODO: Create noise, wake up other nearby monsters.
        if let Some(shout) = self.ecs().brain.get(e).map(|b| b.shout) {
            match shout {
                ShoutType::Shout => {
                    msg!("[One] shout[s] angrily."; self.subject(e));
                }
                ShoutType::Hiss => {
                    msg!("[One] hiss[es]."; self.subject(e));
                }
                ShoutType::Buzz => {
                    msg!("[One] buzz[es] loudly."; self.subject(e));
                }
                ShoutType::Roar => {
                    msg!("[One] roar[s] ferociously."; self.subject(e));
                }
                ShoutType::Gurgle => {
                    msg!("[One] gurgle[s]."; self.subject(e));
                }
                ShoutType::Bark => {
                    msg!("[One] bark[s]."; self.subject(e));
                }
                ShoutType::Meow => {
                    msg!("[One] meow[s]."; self.subject(e));
                }
                ShoutType::Squeak => {
                    msg!("[One] squeak[s]."; self.subject(e));
                }
                ShoutType::Silent => {}
            }
        }
    }

    /// Return whether the entity is a mobile object (eg. active creature).
    pub fn is_mob(&self, e: Entity) -> bool { self.ecs().brain.contains(e) }

    /// Return the AI state of an entity.
    fn brain_state(&self, e: Entity) -> Option<BrainState> {
        self.ecs().brain.get(e).and_then(|brain| Some(brain.state))
    }

    /// Return the value for how a mob will react to other mobs.
    pub fn alignment(&self, e: Entity) -> Option<Alignment> {
        self.ecs().brain.get(e).map(|b| b.alignment)
    }

    /// Return how many frames the entity will delay after an action.
    pub(crate) fn action_delay(&self, e: Entity) -> u32 {
        // Granular speed system:
        // | slow and slowed  | 1 |
        // | slow or slowed   | 2 |
        // | normal           | 3 |
        // | quick or hasted  | 4 |
        // | quick and hasted | 5 |

        let mut speed = 3;
        if self.has_intrinsic(e, Intrinsic::Slow) {
            speed -= 1;
        }
        if self.has_status(e, Status::Slowed) {
            speed -= 1;
        }
        if self.has_intrinsic(e, Intrinsic::Quick) {
            speed += 1;
        }
        if self.has_status(e, Status::Hasted) {
            speed += 1;
        }

        match speed {
            1 => 36,
            2 => 18,
            3 => 12,
            4 => 9,
            5 => 7,
            _ => panic!("Invalid speed value {}", speed),
        }
    }

    /// Return if the entity is a mob that should get an update this frame
    /// based on its speed properties. Does not check for status effects like
    /// sleep that might prevent actual action.
    pub fn ticks_this_frame(&self, e: Entity) -> bool {
        if !self.is_mob(e) || !self.is_alive(e) {
            return false;
        }

        if self.has_status(e, Status::Delayed) {
            return false;
        }

        true
    }

    /// Return whether the entity is dead and should be removed from the world.
    pub fn is_alive(&self, e: Entity) -> bool { self.location(e).is_some() }

    /// Return whether an entity is under computer control
    pub fn is_npc(&self, e: Entity) -> bool { self.is_mob(e) && !self.is_player(e) }

    /// Return whether an entity is the player avatar mob.
    pub fn is_player(&self, e: Entity) -> bool {
        // TODO: Should this just check self.flags.player?
        self.brain_state(e) == Some(BrainState::PlayerControl) && self.is_alive(e)
    }

    /// Return whether the entity is an awake mob.
    pub fn is_active(&self, e: Entity) -> bool {
        match self.brain_state(e) {
            Some(BrainState::Asleep) => false,
            Some(_) => true,
            _ => false,
        }
    }

    /// Return whether the entity is a mob that will act this frame.
    pub fn acts_this_frame(&self, e: Entity) -> bool {
        if !self.is_active(e) {
            return false;
        }
        self.ticks_this_frame(e)
    }

    pub fn player_can_act(&self) -> bool {
        if let Some(p) = self.player() {
            self.acts_this_frame(p)
        } else {
            false
        }
    }

    /// Return whether the entity wants to fight the other entity.
    pub fn is_hostile_to(&self, npc: Entity, other: Entity) -> bool {
        if !self.is_alive(other) {
            // Stop! He's already dead.
            return false;
        }

        if let Some(BrainState::Hunting(target)) = self.brain_state(npc) {
            if npc == target {
                // Already beating him up, obviously he must've done something bad to make you
                // fight him.
                return true;
            }
        }

        let (a, b) = (self.alignment(npc), self.alignment(other));
        if a.is_none() || b.is_none() {
            return false;
        }

        a != b
    }

    /// Look for targets to shoot in a direction.
    pub fn find_ranged_target(&self, shooter: Entity, dir: Dir6, range: usize) -> Option<Entity> {
        let origin = self.location(shooter).unwrap();
        let mut loc = origin;
        for _ in 1..=range {
            loc = loc.jump(self, dir);
            if self.terrain(loc).blocks_shot() {
                break;
            }
            if let Some(e) = self.mob_at(loc) {
                if self.is_hostile_to(shooter, e) {
                    return Some(e);
                }
            }
        }
        None
    }

    /// Try to get the next step on the path from origin towards destination.
    ///
    /// Tries to be fast, not necessarily doing proper pathfinding.
    pub(crate) fn pathing_dir_towards(&self, e: Entity, destination: Location) -> Option<Dir6> {
        // Could do all sorts of cool things here eventually like a Dijkstra map cache, but for now
        // just doing very simple stuff.
        if let Some(origin) = self.location(e) {
            if let Some(dir) = origin.dir6_towards(destination) {
                // Try direct approach, the the other directions.
                for &turn in &[0, 1, -1, 2, -2, 3] {
                    let dir = dir + turn;
                    let next_loc = origin.jump(self, dir);
                    if self.can_enter(e, next_loc) {
                        return Some(dir);
                    }
                }
                return None;
            }
        }
        None
    }

    /// Return whether the entity should have an idle animation.
    pub fn is_bobbing(&self, e: Entity) -> bool { self.is_active(e) && !self.is_player(e) }

    /// Return the set of mobs that are in update range.
    ///
    /// In a large game world, the active set is limited to the player's surroundings.
    pub fn active_mobs(&self) -> Vec<Entity> {
        self.entities()
            .filter(|&&e| self.is_mob(e))
            .cloned()
            .collect()
    }
}
