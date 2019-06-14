use crate::item::Slot;
use crate::mutate::Mutate;
use crate::query::Query;
use crate::world::World;
use crate::seed::Seed;
use calx::Dir6;
use calx::Incremental;
use serde_derive::{Deserialize, Serialize};

/// Return type for actions that might fail.
///
/// Used for early exit with ?-operator in the action functions.
pub type ActionOutcome = Option<()>;

/// Player command events that the world is updated with.
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Called to update the state on frames where the player can't act.
    Wait,
    /// Do nothing, skip your turn.
    Pass,
    /// Take a step in direction.
    Step(Dir6),
    /// Melee attack in direction.
    Melee(Dir6),
    /// Pick up the topmost item from the floor where you're standing on.
    ///
    /// TODO: Item selection support.
    Take,
    /// Drop an item from inventory slot.
    Drop(Slot),
    /// Equip or unequip an item in slot.
    ///
    /// Items in equipment slots are unequipped to inventory. Items in inventory slots are equipped
    /// to the appropriate equipment slot.
    Equip(Slot),
    /// Use a nontargeted inventory item.
    UseItem(Slot),
    /// Use a directionally targeted inventory item.
    Zap(Slot, Dir6),
}

impl Incremental for World {
    type Seed = Seed;
    type Event = Command;

    fn from_seed(s: &Self::Seed) -> Self { World::new(s) }

    fn update(&mut self, e: &Command) {
        if self.player_can_act() {
            debug_assert!(*e != Command::Wait, "Calling wait during player's turn");
            self.clear_events();
            self.process_cmd(e);
        } else {
            debug_assert!(*e == Command::Wait, "Giving inputs outside player's turn");
        }

        self.next_tick();
    }
}

impl World {
    fn process_cmd(&mut self, cmd: &Command) -> ActionOutcome {
        use Command::*;
        match cmd {
            Wait => {
                Some(())
            }
            Pass => {
                let player = self.player()?;
                self.idle(player)
            }
            Step(dir) => {
                let player = self.player()?;
                self.entity_step(player, *dir)
            }
            Melee(dir) => {
                let player = self.player()?;
                self.entity_melee(player, *dir)
            }
            Take => {
                let player = self.player()?;
                let item = self.item_at(self.location(player)?)?;
                self.entity_take(player, item)
            }
            Drop(slot) => {
                let player = self.player()?;
                self.place_entity(self.entity_equipped(player, *slot)?, self.location(player)?);
                Some(())
            }
            Equip(slot) => {
                let player = self.player()?;
                let item = self.entity_equipped(player, *slot)?;
                let swap_slot = if slot.is_equipment_slot() {
                    // Remove equipped.
                    // TODO: Items that can't be removed because of curses etc. trip here.
                    self.free_bag_slot(player)?
                } else {
                    // Equip from bag.
                    // TODO: Inability to equip item because stats limits etc. trips here.
                    self.free_equip_slot(player, item)?
                };

                self.equip_item(item, player, swap_slot);
                Some(())
            }
            UseItem(slot) => {
                let player = self.player()?;
                let item = self.entity_equipped(player, *slot)?;
                let location = self.location(player)?;
                if self.uses_left(item) > 0 {
                    self.drain_charge(item);
                    self.cast_spell(location, item, Some(player))
                } else {
                    msg!(self, "Nothing happens.").send();
                    None
                }
            }
            Zap(slot, dir) => {
                let player = self.player()?;
                let item = self.entity_equipped(player, *slot)?;
                let location = self.location(player)?;
                self.cast_directed_spell(location, *dir, item, Some(player))
            }
        }
    }
}
