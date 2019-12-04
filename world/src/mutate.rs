//! Gameplay logic that changes things

use crate::{
    ai::Brain,
    effect::{Damage, Effect},
    sector::SECTOR_WIDTH,
    stats::Status,
    volume::Volume,
    Ability, ActionOutcome, Anim, AnimState, Ecs, Event, ExternalEntity,
    Location, Slot, World,
};
use calx::{Dir6, RngExt};
use calx_ecs::Entity;
use rand::seq::SliceRandom;
use rand::Rng;

/// World-mutating methods that are not exposed outside the crate.
impl World {
    /// Advance world state after player input has been received.
    pub(crate) fn next_tick(&mut self) {
        self.generate_world_spawns();
        self.tick_anims();

        self.ai_main();

        self.clean_dead();
        self.flags.tick += 1;

        // Expiring entities (animation effects) disappear if their time is up.
        let es: Vec<Entity> = self.ecs.anim.ent_iter().cloned().collect();
        for e in es.into_iter() {
            if let Some(anim) = self.anim(e) {
                if anim
                    .anim_done_world_tick
                    .map(|t| t <= self.get_tick())
                    .unwrap_or(false)
                {
                    self.kill_entity(e);
                }
            }
        }
    }

    pub(crate) fn equip_item(&mut self, e: Entity, parent: Entity, slot: Slot) {
        self.spatial.equip(e, parent, slot);
        self.rebuild_stats(parent);
    }

    pub(crate) fn set_player(&mut self, player: Option<Entity>) {
        self.flags.player = player;
    }

    /// Mark an entity as dead, but don't remove it from the system yet.
    pub(crate) fn kill_entity(&mut self, e: Entity) {
        if self.count(e) > 1 {
            self.ecs_mut().stacking[e].count -= 1;
        } else {
            self.spatial.remove(e);
        }
    }

    /// Remove an entity from the system.
    ///
    /// You generally do not want to call this directly. Mark the entity as dead and it will be
    /// removed at the end of the turn.
    pub(crate) fn remove_entity(&mut self, e: Entity) { self.ecs.remove(e); }

    /// Compute field-of-view into entity's map memory.
    ///
    /// Does nothing for entities without a map memory component.
    pub(crate) fn do_fov(&mut self, e: Entity) {
        if !self.ecs.map_memory.contains(e) {
            return;
        }

        if let Some(origin) = self.location(e) {
            const DEFAULT_FOV_RANGE: i32 = 7;
            const OVERLAND_FOV_RANGE: i32 = SECTOR_WIDTH;

            // Long-range sight while in overworld.
            let range = if self.is_underground(origin) {
                DEFAULT_FOV_RANGE
            } else {
                OVERLAND_FOV_RANGE
            };

            let fov = self.fov_from(origin, range);

            let memory = &mut self.ecs.map_memory[e];
            memory.seen.clear();

            for &loc in &fov {
                memory.seen.insert(loc);
                memory.remembered.insert(loc);
            }
        }
    }

    /// Push an event to the event queue for this tick.
    pub(crate) fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Access the persistent random number generator.
    pub(crate) fn rng(&mut self) -> &mut crate::Rng { &mut self.rng }

    /// Mutable access to ecs
    pub(crate) fn ecs_mut(&mut self) -> &mut Ecs { &mut self.ecs }

    /// Spawn an effect entity
    pub(crate) fn spawn_fx(
        &mut self,
        loc: Location,
        state: AnimState,
    ) -> Entity {
        let e = self.ecs.make();
        self.place_entity(e, loc);

        let mut anim = Anim::default();
        debug_assert!(state.is_transient_anim_state());
        anim.state = state;
        anim.anim_start = self.get_anim_tick();

        // Set the (world clock, not anim clock to preserve determinism) time when animation entity
        // should be cleaned up.
        // XXX: Animations stick around for a bunch of time after becoming spent and invisible,
        // simpler than trying to figure out precise durations.
        anim.anim_done_world_tick = Some(self.get_tick() + 300);

        self.ecs.anim.insert(e, anim);
        e
    }

    /// Remove destroyed entities from system
    pub(crate) fn clean_dead(&mut self) {
        let kill_list: Vec<Entity> = self
            .entities()
            .filter(|&&e| !self.is_alive(e))
            .cloned()
            .collect();

        for e in kill_list {
            self.remove_entity(e);
        }
    }

    pub(crate) fn place_entity(&mut self, e: Entity, mut loc: Location) {
        if self.is_item(e) {
            loc = self.empty_item_drop_location(loc);
        }
        self.set_entity_location(e, loc);
        self.after_entity_moved(e);
    }

    pub(crate) fn after_entity_moved(&mut self, e: Entity) { self.do_fov(e); }

    ////////////////////////////////////////////////////////////////////////////////
    // High-level commands, actual action can change because of eg. confusion.

    pub(crate) fn entity_step(
        &mut self,
        e: Entity,
        dir: Dir6,
    ) -> ActionOutcome {
        if self.confused_move(e) {
            Some(true)
        } else {
            self.really_step(e, dir)
        }
    }

    pub(crate) fn entity_melee(
        &mut self,
        e: Entity,
        dir: Dir6,
    ) -> ActionOutcome {
        if self.confused_move(e) {
            Some(true)
        } else {
            self.really_melee(e, dir)
        }
    }

    /// The entity spends its action waiting.
    pub(crate) fn idle(&mut self, e: Entity) -> ActionOutcome {
        if self.consume_nutrition(e) {
            if let Some(regen) = self.tick_regeneration(e) {
                self.push_event(Event::Damage {
                    entity: e,
                    amount: -regen,
                });
            }
        }
        self.end_turn(e);
        Some(true)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn really_step(
        &mut self,
        e: Entity,
        dir: Dir6,
    ) -> ActionOutcome {
        let origin = self.location(e)?;
        let loc = origin.jump(self, dir);
        if self.can_enter(e, loc) {
            self.place_entity(e, loc);

            let delay = self.action_delay(e);
            debug_assert!(delay > 0);
            let anim_tick = self.get_anim_tick();
            if let Some(anim) = self.ecs_mut().anim.get_mut(e) {
                anim.tween_from = origin;
                anim.tween_start = anim_tick;
                anim.tween_duration = delay;
            }
            self.end_turn(e);
            return Some(true);
        }

        None
    }

    /// Randomly make a confused mob move erratically.
    ///
    /// Return true if confusion kicked in.
    pub(crate) fn confused_move(&mut self, e: Entity) -> bool {
        const CONFUSE_CHANCE_ONE_IN: u32 = 3;

        if !self.has_status(e, Status::Confused) {
            return false;
        }

        if self.rng().one_chance_in(CONFUSE_CHANCE_ONE_IN) {
            let dir = self.rng().gen();
            let loc = if let Some(loc) = self.location(e) {
                loc
            } else {
                return false;
            };
            let destination = loc.jump(self, dir);

            if self.mob_at(destination).is_some() {
                let _ = self.really_melee(e, dir);
            } else {
                let _ = self.really_step(e, dir);
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn spawn(
        &mut self,
        entity: &ExternalEntity,
        loc: Location,
    ) -> Entity {
        let e = self.inject(entity);
        self.place_entity(e, loc);
        e
    }

    /// Special method for setting the player start position.
    ///
    /// If player already exists and is placed in the world, do nothing.
    ///
    /// If player exists, but is not placed in spatial, teleport the player here.
    ///
    /// If player does not exist, create the initial player entity.
    pub(crate) fn spawn_player(
        &mut self,
        loc: Location,
        spec: &ExternalEntity,
    ) {
        if let Some(player) = self.player() {
            if self.location(player).is_none() {
                // Teleport player from limbo.
                self.place_entity(player, loc);
            }
        } else {
            let player = self.inject(&spec);
            // Playerify with the boring component stuff.
            self.ecs_mut().brain.insert(player, Brain::player());
            self.ecs_mut().map_memory.insert(player, Default::default());
            self.set_player(Some(player));
            self.place_entity(player, loc);
        }
    }

    pub(crate) fn apply_effect_to_entity(
        &mut self,
        effect: &Effect,
        target: Entity,
        source: Option<Entity>,
    ) {
        use crate::effect::Effect::*;
        match *effect {
            Hit { amount, damage } => {
                self.damage(target, amount as i32, damage, source);
            }
            Confuse => {
                self.gain_status(target, Status::Confused, 40);
                msg!(self, "[One] [is] confused.").subject(target).send();
            }
        }
    }

    pub(crate) fn apply_effect_to(
        &mut self,
        effect: &Effect,
        loc: Location,
        source: Option<Entity>,
    ) {
        if let Some(mob) = self.mob_at(loc) {
            self.apply_effect_to_entity(effect, mob, source);
        }
    }

    pub(crate) fn apply_effect(
        &mut self,
        effect: &Effect,
        volume: &Volume,
        source: Option<Entity>,
    ) {
        for loc in &volume.0 {
            self.apply_effect_to(effect, *loc, source);
        }
    }

    pub(crate) fn drain_charge(&mut self, item: Entity) {
        if self.destroy_after_use(item) {
            self.kill_entity(item);
        }

        if let Some(i) = self.ecs_mut().item.get_mut(item) {
            if i.charges > 0 {
                i.charges -= 1;
            }
        }
    }

    /// Run autonomous updates on entity that happen each turn
    ///
    /// This runs regardless of the action speed or awakeness status of the entity. The exact same
    /// is run for player and AI entities.
    pub(crate) fn heartbeat(&mut self, e: Entity) { self.tick_statuses(e); }

    pub(crate) fn use_ability(
        &mut self,
        _e: Entity,
        _a: Ability,
    ) -> ActionOutcome {
        // TODO
        None
    }

    pub(crate) fn use_item_ability(
        &mut self,
        e: Entity,
        item: Entity,
        a: Ability,
    ) -> ActionOutcome {
        debug_assert!(!a.is_targeted());
        // TODO: Lift to generic ability use method
        if !self.has_ability(item, a) {
            return None;
        }
        let origin = self.location(e)?;

        match a {
            Ability::LightningBolt => {
                const LIGHTNING_RANGE: u32 = 4;
                const LIGHTNING_EFFECT: Effect = Effect::Hit {
                    amount: 12,
                    damage: Damage::Electricity,
                };

                // TODO: Make an API, more efficient lookup of entities within an area

                let targets: Vec<Entity> = self
                    .sphere_volume(origin, LIGHTNING_RANGE)
                    .0
                    .into_iter()
                    .flat_map(|loc| self.entities_at(loc))
                    .filter(|&x| self.is_mob(x) && x != e)
                    .collect();

                if let Some(target) = targets.choose(self.rng()) {
                    msg!(self, "There is a peal of thunder.").send();
                    let loc = self.location(*target).unwrap();
                    self.apply_effect(
                        &LIGHTNING_EFFECT,
                        &Volume::point(loc),
                        Some(e),
                    );
                } else {
                    msg!(self, "The spell fizzles.").send();
                }
            }
            _ => {
                msg!(self, "TODO cast untargeted spell {:?}", a).send();
            }
        }
        self.drain_charge(item);
        Some(true)
    }

    pub(crate) fn use_targeted_ability(
        &mut self,
        _e: Entity,
        _a: Ability,
        _dir: Dir6,
    ) -> ActionOutcome {
        // TODO
        None
    }

    pub(crate) fn use_targeted_item_ability(
        &mut self,
        e: Entity,
        item: Entity,
        a: Ability,
        dir: Dir6,
    ) -> ActionOutcome {
        debug_assert!(a.is_targeted());
        if !self.has_ability(item, a) {
            return None;
        }
        let origin = self.location(e)?;

        // TODO: Lift to generic ability use method

        match a {
            Ability::Fireball => {
                const FIREBALL_RANGE: u32 = 9;
                const FIREBALL_RADIUS: u32 = 1;
                const FIREBALL_EFFECT: Effect = Effect::Hit {
                    amount: 6,
                    damage: Damage::Fire,
                };
                let center = self.projected_explosion_center(
                    origin,
                    dir,
                    FIREBALL_RANGE,
                );
                let volume = self.sphere_volume(center, FIREBALL_RADIUS);
                self.apply_effect(&FIREBALL_EFFECT, &volume, Some(e));

                // TODO: Maybe move anim generation to own procedure?
                const PROJECTILE_TIME: u64 = 8;
                for &pt in &volume.0 {
                    let fx = self.spawn_fx(pt, AnimState::Explosion);
                    self.anim_mut(fx).unwrap().anim_start += PROJECTILE_TIME;
                }

                let anim_tick = self.get_anim_tick();
                let projectile = self.spawn_fx(center, AnimState::Firespell);
                {
                    let anim = self.anim_mut(projectile).unwrap();
                    anim.tween_from = origin;
                    anim.tween_start = anim_tick;
                    anim.tween_duration = PROJECTILE_TIME as u32;
                }
            }
            Ability::Confuse => {
                const CONFUSION_RANGE: u32 = 9;

                let center = self.projected_explosion_center(
                    origin,
                    dir,
                    CONFUSION_RANGE,
                );
                self.apply_effect(
                    &Effect::Confuse,
                    &Volume::point(center),
                    Some(e),
                );
            }
            _ => {
                msg!(self, "TODO cast directed spell {:?}", a).send();
            }
        }
        self.drain_charge(item);
        Some(true)
    }
}
