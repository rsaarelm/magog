use calx_ecs::Entity;
use calx_grid::Dir6;
use location::Location;
use query::Query;
use command::CommandResult;
use terraform::Terraform;
use world::Loadout;
use form;

/// World-mutating methods that are not exposed outside the crate.
pub trait Mutate: Query + Terraform + Sized {
    /// Advance world state after player input has been received.
    ///
    /// Returns CommandResult Ok(()) so can used to end result-returning methods.
    fn next_tick(&mut self) -> CommandResult;

    fn set_entity_location(&mut self, e: Entity, loc: Location);

    /// Run AI for all autonomous mobs.
    fn ai_main(&mut self) {
        unimplemented!();
    }

    /// Remove destroyed entities from system
    fn clean_dead(&mut self) {
        unimplemented!();
    }

    fn place_entity(&mut self, e: Entity, loc: Location) {
        self.set_entity_location(e, loc);
        self.after_entity_moved(e);
    }

    fn after_entity_moved(&mut self, e: Entity) {
        // TODO
    }

    fn entity_step(&mut self, e: Entity, dir: Dir6) -> CommandResult {
        let loc = try!(self.location(e).ok_or(()));
        if self.can_enter(e, loc) {
            self.place_entity(e, loc);
        }

        Err(())
    }

    fn entity_melee(&mut self, e: Entity, dir: Dir6) -> CommandResult {
        unimplemented!();
    }

    fn do_fov(&mut self, e: Entity) {
        unimplemented!();
    }

    fn init_level(&mut self, depth: u32) {
        if let None = self.player() {
            // XXX: Assuming player is the first element
            // TODO: A spawn named method instead.
            let player = form::FORMS[0].build(self);
            self.place_entity(player, Location::new(0, 0, 0));
        }

        // TODO: Map init.
    }

    fn spawn(&mut self, loadout: &Loadout) -> Entity;
}
