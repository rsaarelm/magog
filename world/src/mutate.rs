use calx::{Dir6, RngExt};
use calx_ecs::Entity;
use crate::command::CommandResult;
use crate::components::{Brain, BrainState, MapMemory, Status};
use crate::effect::{Damage, Effect};
use crate::event::Event;
use crate::item::{ItemType, MagicEffect, Slot};
use crate::location::Location;
use crate::mapsave;
use crate::query::Query;
use crate::spec;
use crate::terraform::Terraform;
use crate::volume::Volume;
use crate::world::{Ecs, Loadout};
use crate::Distribution;
use crate::{attack_damage, roll};
use rand::{seq, Rng};

/// World-mutating methods that are not exposed outside the crate.
pub trait Mutate: Query + Terraform + Sized {
    /// Advance world state after player input has been received.
    ///
    /// Returns CommandResult so can used to end result-returning methods.
    fn next_tick(&mut self) -> CommandResult;

    /// Advance animations without ticking the world logic.
    ///
    /// Use this when waiting for player input to finish pending animations.
    fn tick_anims(&mut self) {
        let entities: Vec<Entity> = self.entities().cloned().collect();
        for e in entities {
            self.ecs_mut().anim.get_mut(e).map(|a| a.tick());
        }
    }

    fn set_entity_location(&mut self, e: Entity, loc: Location);

    fn equip_item(&mut self, e: Entity, parent: Entity, slot: Slot);

    fn set_player(&mut self, player: Option<Entity>);

    /// Mark an entity as dead, but don't remove it from the system yet.
    fn kill_entity(&mut self, e: Entity);

    /// Remove an entity from the system.
    ///
    /// You generally do not want to call this directly. Mark the entity as dead and it will be
    /// removed at the end of the turn.
    fn remove_entity(&mut self, e: Entity);

    /// Compute field-of-view into entity's map memory.
    ///
    /// Does nothing for entities without a map memory component.
    fn do_fov(&mut self, e: Entity);

    /// Push an event to the event queue for this tick.
    fn push_event(&mut self, event: Event);

    /// Access the persistent random number generator.
    fn rng(&mut self) -> &mut crate::Rng;

    /// Mutable access to ecs
    fn ecs_mut(&mut self) -> &mut Ecs;

    /// Run AI for all autonomous mobs.
    fn ai_main(&mut self) {
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
        const WAKEUP_DISTANCE: i32 = 5;

        use crate::components::BrainState::*;
        let brain_state = self.brain_state(npc).expect("Running AI for non-mob");
        match brain_state {
            Asleep => {
                // XXX: Only treat player mob as potential hostile.
                // Can't model area conflict effects yet.
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
                            self.designate_enemy(npc, player);
                        }
                    }
                }
            }
            Hunting(target) => {
                if let (Some(my_loc), Some(target_loc)) =
                    (self.location(npc), self.location(target))
                {
                    if my_loc.metric_distance(target_loc) == 1 {
                        let _ = self.entity_melee(npc, my_loc.dir6_towards(target_loc).unwrap());
                    } else if let Some(move_dir) = self.pathing_dir_towards(npc, target_loc) {
                        let _ = self.entity_step(npc, move_dir);
                    }
                }
            }
            PlayerControl => {}
        }
    }

    /// End move for entity.
    ///
    /// Applies delay.
    fn end_turn(&mut self, e: Entity) {
        let delay = self.action_delay(e);
        self.gain_status(e, Status::Delayed, delay);
    }

    fn notify_attacked_by(&mut self, victim: Entity, attacker: Entity) {
        // TODO: Check if victim is already in close combat and don't disengage against new target
        // if it is.
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
    fn shout(&mut self, e: Entity) {
        // TODO: Create noise, wake up other nearby monsters.
        use crate::components::ShoutType;
        if let Some(shout) = self.ecs().brain.get(e).map(|b| b.shout) {
            match shout {
                ShoutType::Shout => {
                    msg!(self, "[One] shout[s] angrily.").subject(e).send();
                }
                ShoutType::Hiss => {
                    msg!(self, "[One] hiss[es] angrily.").subject(e).send();
                }
                ShoutType::Buzz => {
                    msg!(self, "[One] buzz[es] loudly.").subject(e).send();
                }
                ShoutType::Roar => {
                    msg!(self, "[One] roar[s] angrily.").subject(e).send();
                }
                ShoutType::Gurgle => {
                    msg!(self, "[One] gurgle[s] loudly.").subject(e).send();
                }
                ShoutType::Silent => {}
            }
        }
    }

    /// Remove destroyed entities from system
    fn clean_dead(&mut self) {
        let kill_list: Vec<Entity> = self
            .entities()
            .filter(|&&e| !self.is_alive(e))
            .cloned()
            .collect();

        for e in kill_list {
            self.remove_entity(e);
        }
    }

    fn place_entity(&mut self, e: Entity, mut loc: Location) {
        if self.is_item(e) {
            loc = self.empty_item_drop_location(loc);
        }
        self.set_entity_location(e, loc);
        self.after_entity_moved(e);
    }

    fn after_entity_moved(&mut self, e: Entity) { self.do_fov(e); }

    ////////////////////////////////////////////////////////////////////////////////
    // High-level commands, actual action can change because of eg. confusion.

    fn entity_step(&mut self, e: Entity, dir: Dir6) -> Result<(), ()> {
        if self.confused_move(e) {
            Ok(())
        } else {
            self.really_step(e, dir)
        }
    }

    fn entity_melee(&mut self, e: Entity, dir: Dir6) -> Result<(), ()> {
        if self.confused_move(e) {
            Ok(())
        } else {
            self.really_melee(e, dir)
        }
    }

    fn entity_take(&mut self, e: Entity, item: Entity) -> Result<(), ()> {
        // Only mobs can take items.
        if !self.is_mob(e) {
            return Err(());
        }

        if !self.is_item(item) {
            return Err(());
        }

        // Somehow trying to pick up something we're inside of. Pls don't break the universe.
        if self.entity_contains(item, e) {
            panic!("Trying to pick up an entity you are inside of. This shouldn't happen");
        }

        if let Some(slot) = self.free_bag_slot(e) {
            self.equip_item(item, e, slot);
            if self.is_player(e) {
                msg!(self, "[One] pick[s] up [another].")
                    .subject(e)
                    .object(item)
                    .send();
            }

            self.end_turn(e);
            Ok(())
        } else {
            // No more inventory space
            Err(())
        }
    }

    /// Cast an undirected spell
    fn cast_spell(
        &mut self,
        origin: Location,
        effect: Entity,
        caster: Option<Entity>,
    ) -> Result<(), ()> {
        if let ItemType::UntargetedUsable(effect) = self.ecs().item.get(effect).ok_or(())?.item_type
        {
            match effect {
                MagicEffect::Lightning => {
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
                        .filter(|&e| self.is_mob(e) && Some(e) != caster)
                        .collect();

                    let mut target = seq::sample_iter(self.rng(), &targets, 1).unwrap();

                    if let Some(target) = target.pop() {
                        msg!(self, "There is a peal of thunder.").send();
                        let loc = self.location(*target).unwrap();
                        self.apply_effect(&LIGHTNING_EFFECT, &Volume::point(loc), caster);
                    } else {
                        msg!(self, "The spell fizzles.").send();
                    }
                }
                _ => {
                    msg!(self, "TODO cast untargeted spell {:?}", effect).send();
                }
            }
            caster.map(|e| self.end_turn(e));
            Ok(())
        } else {
            Err(())
        }
    }

    /// Cast a directed spell
    fn cast_directed_spell(
        &mut self,
        origin: Location,
        dir: Dir6,
        effect: Entity,
        caster: Option<Entity>,
    ) -> Result<(), ()> {
        if let ItemType::TargetedUsable(effect) = self.ecs().item.get(effect).ok_or(())?.item_type {
            match effect {
                MagicEffect::Fireball => {
                    const FIREBALL_RANGE: u32 = 9;
                    const FIREBALL_RADIUS: u32 = 2;
                    const FIREBALL_EFFECT: Effect = Effect::Hit {
                        amount: 6,
                        damage: Damage::Fire,
                    };
                    let center = self.projected_explosion_center(origin, dir, FIREBALL_RANGE);
                    let volume = self.sphere_volume(center, FIREBALL_RADIUS);
                    self.apply_effect(&FIREBALL_EFFECT, &volume, caster);
                }
                MagicEffect::Confuse => {
                    const CONFUSION_RANGE: u32 = 9;

                    let center = self.projected_explosion_center(origin, dir, CONFUSION_RANGE);
                    self.apply_effect(&Effect::Confuse, &Volume::point(center), caster);
                }
                _ => {
                    msg!(self, "TODO cast directed spell {:?}", effect).send();
                }
            }
            caster.map(|e| self.end_turn(e));
            Ok(())
        } else {
            Err(())
        }
    }

    /// The entity spends its action waiting.
    fn idle(&mut self, e: Entity) {
        if self.consume_nutrition(e) {
            if let Some(regen) = self.tick_regeneration(e) {
                self.push_event(Event::Damage {
                    entity: e,
                    amount: -regen,
                });
            }
        }
        self.end_turn(e);
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn really_step(&mut self, e: Entity, dir: Dir6) -> Result<(), ()> {
        let origin = self.location(e).ok_or(())?;
        let loc = origin.jump(self, dir);
        if self.can_enter(e, loc) {
            self.place_entity(e, loc);

            let delay = self.action_delay(e);
            debug_assert!(delay > 0);
            if let Some(anim) = self.ecs_mut().anim.get_mut(e) {
                anim.tween_from = origin;
                anim.tween_current = delay - 1;
                anim.tween_max = delay;
            }
            self.end_turn(e);
            return Ok(());
        }

        Err(())
    }

    fn really_melee(&mut self, e: Entity, dir: Dir6) -> Result<(), ()> {
        if let Some(loc) = self.location(e) {
            if let Some(target) = self.mob_at(loc.jump(self, dir)) {
                // XXX: Using power stat for damage, should this be different?
                // Do +5 since dmg 1 is really, really useless.
                let advantage = self.stats(e).attack - self.stats(target).defense
                    + 2 * self.stats(target).armor;
                let damage = attack_damage(roll(self.rng()), advantage, 5 + self.stats(e).power);

                if damage == 0 {
                    msg!(self, "[One] miss[es] [another].")
                        .subject(e)
                        .object(target)
                        .send();
                } else {
                    msg!(self, "[One] hit[s] [another] for {}.", damage)
                        .subject(e)
                        .object(target)
                        .send();
                }
                self.damage(target, damage, Damage::Physical, Some(e));
                self.end_turn(e);
                return Ok(());
            }
        }
        Err(())
    }

    /// Randomly make a confused mob move erratically.
    ///
    /// Return true if confusion kicked in.
    fn confused_move(&mut self, e: Entity) -> bool {
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

    fn damage(&mut self, e: Entity, amount: i32, damage_type: Damage, source: Option<Entity>) {
        if let Some(attacker) = source {
            self.notify_attacked_by(e, attacker);
        }

        let max_hp = self.max_hp(e);

        let mut kill = false;
        if let Some(health) = self.ecs_mut().health.get_mut(e) {
            if amount > 0 {
                health.wounds += amount;

                if health.wounds > max_hp {
                    kill = true;
                }
            }
        }

        if kill {
            if let Some(loc) = self.location(e) {
                if self.player_sees(loc) {
                    // TODO: message templating
                    msg!(
                        self,
                        "[One] {}.",
                        match damage_type {
                            Damage::Physical => "die[s]",
                            Damage::Fire => "burn[s] to ash",
                            Damage::Electricity => "[is] electrocuted",
                            Damage::Cold => "shatter[s] to frozen pieces",
                        }
                    ).subject(e)
                    .send();
                }
            }
            self.kill_entity(e);
        }
    }

    /// Do a single step of natural regeneration for a creature.
    ///
    /// Return amount of health gained, or None if at full health.
    fn tick_regeneration(&mut self, e: Entity) -> Option<i32> {
        let max_hp = self.max_hp(e);
        let increase = (max_hp / 30).max(1);

        if let Some(health) = self.ecs_mut().health.get_mut(e) {
            if health.wounds > 0 {
                let increase = increase.min(health.wounds);
                health.wounds -= increase;
                Some(increase)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn spawn(&mut self, loadout: &Loadout, loc: Location) -> Entity;

    fn deploy_prefab(&mut self, origin: Location, prefab: &mapsave::Prefab) {
        for (&p, &(ref terrain, _)) in prefab.iter() {
            let loc = origin + p;

            // Annihilate any existing entities in the drop zone.
            let es = self.entities_at(loc);
            for &e in &es {
                self.remove_entity(e);
            }

            self.set_terrain(loc, *terrain);
        }

        // Spawn entities after all terrain is in place so that initial FOV is good.
        for (&p, &(_, ref entities)) in prefab.iter() {
            let loc = origin + p;

            for spawn in entities.iter() {
                if spawn == &*spec::PLAYER_SPAWN {
                    self.spawn_player(loc);
                } else {
                    let loadout = spawn.sample(self.rng());
                    self.spawn(&loadout, loc);
                }
            }
        }
    }

    /// Special method for setting the player start position.
    ///
    /// If player already exists and is placed in the world, do nothing.
    ///
    /// If player exists, but is not placed in spatial, teleport the player here.
    ///
    /// If player does not exist, create the initial player entity.
    fn spawn_player(&mut self, loc: Location) {
        if let Some(player) = self.player() {
            if self.location(player).is_none() {
                // Teleport player from limbo.
                self.place_entity(player, loc);
            }
        } else {
            // Initialize new player object. Add some special components you don't get on regular
            // mob spawns.
            let loadout = spec::PLAYER_SPAWN
                .sample(self.rng())
                .c(Brain::player())
                .c(MapMemory::default());
            let player = self.spawn(&loadout, loc);
            self.set_player(Some(player));
        }
    }

    fn apply_effect_to_entity(&mut self, effect: &Effect, target: Entity, source: Option<Entity>) {
        use crate::effect::Effect::*;
        match *effect {
            Heal(_amount) => {
                unimplemented!();
            }
            Hit { amount, damage } => {
                self.damage(target, amount as i32, damage, source);
            }
            Confuse => {
                self.gain_status(target, Status::Confused, 40);
                msg!(self, "[One] [is] confused.").subject(target).send();
            }
            MagicMap => {
                unimplemented!();
            }
        }
    }

    fn apply_effect_to(&mut self, effect: &Effect, loc: Location, source: Option<Entity>) {
        if let Some(mob) = self.mob_at(loc) {
            self.apply_effect_to_entity(effect, mob, source);
        }
    }

    fn apply_effect(&mut self, effect: &Effect, volume: &Volume, source: Option<Entity>) {
        for loc in &volume.0 {
            self.apply_effect_to(effect, *loc, source);
        }
    }

    fn drain_charge(&mut self, item: Entity) {
        let mut emptied = false;
        if let Some(i) = self.ecs_mut().item.get_mut(item) {
            if i.charges > 0 {
                i.charges -= 1;

                if i.charges == 0 {
                    emptied = true;
                }
            }
        }

        if emptied {
            if self.destroy_after_use(item) {
                self.kill_entity(item);
            }
        }
    }

    /// Run autonomous updates on entity that happen each turn
    ///
    /// This runs regardless of the action speed or awakeness status of the entity. The exact same
    /// is run for player and AI entities.
    fn heartbeat(&mut self, e: Entity) { self.tick_statuses(e); }

    fn gain_status(&mut self, e: Entity, status: Status, duration: u32) {
        if duration == 0 {
            return;
        }

        if let Some(statuses) = self.ecs_mut().status.get_mut(e) {
            if let Some(current_duration) = statuses.get(&status).cloned() {
                if duration > current_duration {
                    // Pump up the duration.
                    statuses.insert(status, duration);
                }
            } else {
                // TODO: Special stuff when status first goes into effect goes here
                statuses.insert(status, duration);
            }
        }
    }

    fn tick_statuses(&mut self, e: Entity) {
        if let Some(statuses) = self.ecs_mut().status.get_mut(e) {
            let mut remove = Vec::new();

            for (k, d) in statuses.iter_mut() {
                *d -= 1;
                if *d == 0 {
                    remove.push(*k);
                }
            }

            // TODO: Special stuff when status goes out of effect for dropped statuses.
            for k in remove.into_iter() {
                statuses.remove(&k);
            }
        }
    }

    /// Rebuild cached derived stats of an entity.
    ///
    /// Must be explicitly called any time either the entity's base stats or anything relating to
    /// attached stat-affecting entities like equipped items is changed.
    fn rebuild_stats(&mut self, e: Entity) {
        if !self.ecs().stats.contains(e) {
            return;
        }

        // Start with the entity's base stats.
        let mut stats = self.base_stats(e);

        // Add in stat modifiers from equipped items.
        for &slot in Slot::equipped_iter() {
            if let Some(item) = self.entity_equipped(e, slot) {
                stats = stats + self.stats(item);
            }
        }

        // Set the derived stats.
        self.ecs_mut().stats[e].actual = stats;
    }

    /// Consume one unit of nutrition
    ///
    /// Return false if the entity has an empty stomach.
    fn consume_nutrition(&mut self, _: Entity) -> bool {
        // TODO nutrition system
        true
    }
}
