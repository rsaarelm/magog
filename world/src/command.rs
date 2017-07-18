use calx_ecs::Entity;
use calx_grid::Dir6;
use item::Slot;
use mutate::Mutate;

pub type CommandResult = Result<(), ()>;

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
    fn pass(&mut self) -> CommandResult { self.next_tick() }

    /// Take item from floor
    ///
    /// No selection support yet for multiple items, you pick up the topmost one.
    /// Try to maintain a convention where there's no more than one item in a single location.
    fn take(&mut self) -> CommandResult {
        let player = self.player().ok_or(())?;
        let location = self.location(player).ok_or(())?;
        if let Some(item) = self.item_at(location) {
            self.entity_take(player, item)
        } else {
            Err(())
        }
    }

    /// Drop item held in slot.
    fn drop(&mut self, slot: Slot) -> CommandResult {
        let player = self.player().ok_or(())?;
        unimplemented!();
    }

    /// Swap item between equipment and inventory slots
    ///
    /// Behavior depends on slot. Equipment slots go to inventory, inventory slots go to equip. The
    /// item will be moved to the first available slot.
    fn equip(&mut self, slot: Slot) -> CommandResult {
        let player = self.player().ok_or(())?;
        unimplemented!();
    }

    /// Use a nontargeted effect item.
    fn use_item(&mut self, slot: Slot) -> CommandResult {
        let player = self.player().ok_or(())?;
        unimplemented!();
    }

    /// Use a directionally targeted effect item.
    fn zap_item(&mut self, slot: Slot, dir: Dir6) -> CommandResult {
        let player = self.player().ok_or(())?;
        unimplemented!();
    }
}
