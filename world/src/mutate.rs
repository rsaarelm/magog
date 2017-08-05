use calx_alg::{Deciban, clamp};
use calx_ecs::Entity;
use calx_grid::{Dir6, Prefab};
use command::CommandResult;
use effect::{Damage, Effect};
use event::Event;
use form::Form;
use item::Slot;
use location::Location;
use query::Query;
use rand::Rng;
use terraform::Terraform;
use terrain::Terrain;
use volume::Volume;
use world::{Ecs, Loadout};

/// World-mutating methods that are not exposed outside the crate.
pub trait Mutate: Query + Terraform + Sized {
    /// Advance world state after player input has been received.
    ///
    /// Returns CommandResult Ok(()) so can used to end result-returning methods.
    fn next_tick(&mut self) -> CommandResult;

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
    fn rng(&mut self) -> &mut ::Rng;

    /// Mutable access to ecs
    fn ecs_mut(&mut self) -> &mut Ecs;

    /// Standard deciban roll, convert to i32 and clamp out extremes.
    fn roll(&mut self) -> f32 { clamp(-20.0, 20.0, self.rng().gen::<Deciban>().0.round()) }

    /// Run AI for all autonomous mobs.
    fn ai_main(&mut self) {
        unimplemented!();
    }

    fn notify_attacked_by(&mut self, victim: Entity, attacker: Entity) {
        // TODO: Check if victim is already in close combat and don't disengage against new target
        // if it is.
        self.designate_enemy(victim, attacker);
    }

    fn designate_enemy(&mut self, _e: Entity, _target: Entity) {
        // TODO
    }

    /// Remove destroyed entities from system
    fn clean_dead(&mut self) {
        let kill_list: Vec<Entity> = self.entities()
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

    fn entity_step(&mut self, e: Entity, dir: Dir6) -> Result<(), ()> {
        let loc = try!(self.location(e).ok_or(())) + dir;
        if self.can_enter(e, loc) {
            self.place_entity(e, loc);
            return Ok(());
        }

        Err(())
    }

    fn entity_melee(&mut self, e: Entity, dir: Dir6) -> Result<(), ()> {
        if let Some(loc) = self.location(e) {
            if let Some(target) = self.mob_at(loc + dir) {

                // XXX: Using power stat for damage, should this be different?
                let damage = ::attack_damage(
                    self.roll(),
                    self.stats(e).attack,
                    self.stats(e).power,
                    self.stats(target).defense,
                    self.stats(target).armor,
                );

                self.damage(target, damage, Damage::Physical, Some(e));
                return Ok(());
            }
        }
        Err(())
    }

    fn damage(&mut self, e: Entity, amount: i32, _damage_type: Damage, source: Option<Entity>) {
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
            self.kill_entity(e);
        }
    }

    fn spawn(&mut self, loadout: &Loadout, loc: Location) -> Entity;

    fn deploy_prefab(&mut self, origin: Location, prefab: &Prefab<(Terrain, Vec<String>)>) {
        for (p, &(ref terrain, _)) in prefab.iter() {
            let loc = origin + p;

            // Annihilate any existing entities in the drop zone.
            let es = self.entities_at(loc);
            for &e in &es {
                self.remove_entity(e);
            }

            self.set_terrain(loc, *terrain);
        }

        // Spawn entities after all terrain is in place so that initial FOV is good.
        for (p, &(_, ref entities)) in prefab.iter() {
            let loc = origin + p;

            for spawn in entities.iter() {
                if spawn == "player" {
                    self.spawn_player(loc);
                } else {
                    let form = Form::named(spawn).expect(&format!("Form '{}' not found!", spawn));
                    self.spawn(&form.loadout, loc);
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
            // Initialize new player object.
            let form = Form::named("player").expect("Player form not found");
            let player = self.spawn(&form.loadout, loc);
            self.set_player(Some(player));
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
                msg!(self, "Picked up {}", self.entity_name(item));
            }

            Ok(())
        } else {
            // No more inventory space
            Err(())
        }
    }

    /// Cast an undirected spell
    fn cast_spell(&mut self, _origin: Location, _effect: Entity) {
        // TODO: Probably want a dedicated spell struct here instead of using Entity.
        unimplemented!();
    }

    /// Cast a directed spell
    fn cast_directed_spell(&mut self, _origin: Location, _dir: Dir6, _effect: Entity) {
        // TODO: Probably want a dedicated spell struct here instead of using Entity.
        unimplemented!();
    }

    fn apply_effect_to_entity(&mut self, effect: Effect, target: Entity, source: Option<Entity>) {
        use effect::Effect::*;
        match effect {
            Heal(_amount) => {
                unimplemented!();
            }
            Hit { amount, damage } => {
                self.damage(target, amount as i32, damage, source);
            }
            Confuse => {
                unimplemented!();
            }
            MagicMap => {
                unimplemented!();
            }
        }
    }

    fn apply_effect_to(&mut self, effect: Effect, loc: Location, source: Option<Entity>) {
        if let Some(mob) = self.mob_at(loc) {
            self.apply_effect_to_entity(effect, mob, source);
        }
    }

    fn apply_effect(&mut self, effect: Effect, volume: Volume, source: Option<Entity>) {
        for loc in volume.0 {
            self.apply_effect_to(effect, loc, source);
        }
    }
}
