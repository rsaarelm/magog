use crate::effect::Ability;
use crate::item::Slot;
use crate::mutate::Mutate;
use crate::query::Query;
use crate::sector::WorldSkeleton;
use crate::world::World;
use calx::Dir6;
use calx::Incremental;
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};

/// Return type for actions that might fail.
///
/// Used for early exit with ?-operator in the action functions.
///
/// Return true if the action advances time, false if the player can keep giving commands.
pub type ActionOutcome = Option<bool>;

/// Player command events that the world is updated with.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
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
    /// Place an item in inventory slot.
    InventoryPlace(Entity, Slot),
    /// Swap two slotted items in inventory.
    InventorySwap(Slot, Slot),
    /// Use an undirected action that may be invoked via an item.
    UntargetedAbility {
        ability: Ability,
        item: Option<Entity>,
    },
    /// Use a directed action that may be invoked via an item.
    TargetedAbility {
        ability: Ability,
        dir: Dir6,
        item: Option<Entity>,
    },
}

impl Incremental for World {
    type Seed = u32;
    type Event = Command;

    fn from_seed(s: &Self::Seed) -> Self { World::new(*s, WorldSkeleton::overworld_sprawl()) }

    fn update(&mut self, e: &Command) {
        self.clear_events();
        if self.player_can_act() {
            debug_assert!(*e != Command::Wait, "Calling wait during player's turn");
            self.process_cmd(e);
        } else {
            debug_assert!(*e == Command::Wait, "Giving inputs outside player's turn");
        }

        self.next_tick();
    }
}

impl World {
    /// Return whether a command will work in the current world state.
    ///
    /// Mostly for cases where the feedback is important for the UI (eg. inventory logic).
    pub fn can_command(&self, cmd: &Command) -> bool {
        use Command::*;

        if self.player().is_none() {
            return *cmd == Command::Wait;
        }
        let player = self.player().unwrap();

        match cmd {
            Wait => !self.player_can_act(),

            InventoryPlace(item, slot) => {
                if !self.entity_contains(player, *item) {
                    return false;
                }
                if self.entity_equipped(player, *slot).is_some() {
                    return false;
                }
                if !slot.accepts(self.equip_type(*item)) {
                    return false;
                }
                true
            }

            InventorySwap(slot1, slot2) => {
                if slot1 == slot2 {
                    return false;
                }
                if let Some(e) = self.entity_equipped(player, *slot1) {
                    if !slot2.accepts(self.equip_type(e)) {
                        return false;
                    }
                }

                if let Some(e) = self.entity_equipped(player, *slot2) {
                    if !slot1.accepts(self.equip_type(e)) {
                        return false;
                    }
                }
                true
            }

            // TODO: Add failure checks for the rest as needed.
            _ => true,
        }
    }

    fn process_cmd(&mut self, cmd: &Command) -> ActionOutcome {
        use Command::*;
        match cmd {
            Wait => Some(true),
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
                // Dropping items does not cost a turn since you'll be doing it from the inventory
                // screen.
                Some(false)
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
                Some(false)
            }
            InventoryPlace(item, slot) => {
                // TODO: Check curses, limits etc, as in Equip
                let player = self.player()?;

                // Checks implemented in can_command, piggyback on those.
                if !self.can_command(cmd) {
                    return None;
                }

                self.equip_item(*item, player, *slot);
                Some(false)
            }
            InventorySwap(slot1, slot2) => {
                // TODO: Check curses, limits etc, as in Equip
                let player = self.player()?;

                // Checks implemented in can_command, piggyback on those.
                if !self.can_command(cmd) {
                    return None;
                }

                let e1 = self.entity_equipped(player, *slot1);
                let e2 = self.entity_equipped(player, *slot2);

                if let Some(e1) = e1 {
                    // XXX: Ad hoc "move it out of the way" slot.
                    self.equip_item(e1, player, Slot::Bag(999_999));
                }
                if let Some(e2) = e2 {
                    self.equip_item(e2, player, *slot1);
                }
                if let Some(e1) = e1 {
                    self.equip_item(e1, player, *slot2);
                }

                Some(false)
            }

            UntargetedAbility { ability, item } => {
                // XXX: Should these be asserts or just returns?
                debug_assert!(!ability.is_targeted());

                let player = self.player()?;
                if let Some(item) = item {
                    self.use_item_ability(player, *item, *ability)
                } else {
                    self.use_ability(player, *ability)
                }
            }

            TargetedAbility { ability, dir, item } => {
                debug_assert!(ability.is_targeted());
                let player = self.player()?;
                if let Some(item) = item {
                    self.use_targeted_item_ability(player, *item, *ability, *dir)
                } else {
                    self.use_targeted_ability(player, *ability, *dir)
                }
            }
        }
    }
}
