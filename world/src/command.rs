use calx::Dir6;
use crate::event::Event;
use crate::item::Slot;
use crate::mutate::Mutate;

pub type CommandResult = Result<Vec<Event>, ()>;

/// Player actions.
pub trait Command: Mutate + Sized {
    /// The player tries to step in a direction.
    ///
    /// Will fail if there are any obstacles blocking the path.
    ///
    /// Will fail if the player is incapacitated.
    fn step(&mut self, dir: Dir6) -> CommandResult {
        let player = self.player().ok_or(())?;
        self.entity_step(player, dir)?;
        self.next_tick()
    }

    /// The player performs a melee attack in direction.
    ///
    /// Will fail if the player is incapacitated.
    ///
    /// Melee attacks against empty air are allowed.
    fn melee(&mut self, dir: Dir6) -> CommandResult {
        let player = self.player().ok_or(())?;
        self.entity_melee(player, dir)?;
        self.next_tick()
    }

    /// Pass a turn without action from the player.
    ///
    /// Will usually succeed, but some games might not let the player pass turns.
    fn pass(&mut self) -> CommandResult {
        if let Some(player) = self.player() {
            self.idle(player);
        }
        self.next_tick()
    }

    /// Take item from floor
    ///
    /// No selection support yet for multiple items, you pick up the topmost one.
    /// Try to maintain a convention where there's no more than one item in a single location.
    fn take(&mut self) -> CommandResult {
        let player = self.player().ok_or(())?;
        let location = self.location(player).ok_or(())?;
        if let Some(item) = self.item_at(location) {
            self.entity_take(player, item)?;
            self.next_tick()
        } else {
            Err(())
        }
    }

    /// Drop item held in slot.
    fn drop(&mut self, slot: Slot) -> CommandResult {
        let player = self.player().ok_or(())?;
        let location = self.location(player).ok_or(())?;
        if let Some(item) = self.entity_equipped(player, slot) {
            self.place_entity(item, location);
            self.next_tick()
        } else {
            Err(())
        }
    }

    /// Swap item between equipment and inventory slots
    ///
    /// Behavior depends on slot. Equipment slots go to inventory, inventory slots go to equip. The
    /// item will be moved to the first available slot.
    fn equip(&mut self, slot: Slot) -> CommandResult {
        let player = self.player().ok_or(())?;
        let item = self.entity_equipped(player, slot).ok_or(())?;

        let swap_slot = if slot.is_equipment_slot() {
            // Remove equipped.
            // TODO: Items that can't be removed because of curses etc. trip here.
            self.free_bag_slot(player).ok_or(())?
        } else {
            // Equip from bag.
            // TODO: Inability to equip item because stats limits etc. trips here.
            self.free_equip_slot(player, item).ok_or(())?
        };

        self.equip_item(item, player, swap_slot);
        self.next_tick()
    }

    /// Use a nontargeted effect item.
    fn use_item(&mut self, slot: Slot) -> CommandResult {
        let player = self.player().ok_or(())?;
        let location = self.location(player).ok_or(())?;
        let item = self.entity_equipped(player, slot).ok_or(())?;
        if self.uses_left(item) > 0 {
            self.cast_spell(location, item, Some(player))?;
            self.drain_charge(item);
        } else {
            msg!(self, "Nothing happens.").send();
        }
        self.next_tick()
    }

    /// Use a directionally targeted effect item.
    fn zap_item(&mut self, slot: Slot, dir: Dir6) -> CommandResult {
        let player = self.player().ok_or(())?;
        let location = self.location(player).ok_or(())?;
        let item = self.entity_equipped(player, slot).ok_or(())?;
        self.cast_directed_spell(location, dir, item, Some(player))?;
        self.next_tick()
    }
}
