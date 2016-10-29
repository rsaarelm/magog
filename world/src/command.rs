use calx_grid::Dir6;

type CommandResult = Result<(), ()>;

/// Player actions.
pub trait Command {
    /// The player tries to step in a direction.
    ///
    /// Will fail if there are any obstacles blocking the path.
    ///
    /// Will fail if the player is incapacitated.
    fn step(&mut self, dir: Dir6) -> CommandResult;

    /// The player performs a melee attack in direction.
    ///
    /// Will fail if the player is incapacitated.
    ///
    /// Melee attacks against empty air are allowed.
    fn melee(&mut self, dir: Dir6) -> CommandResult;

    /// Pass a turn without action from the player.
    ///
    /// Will usually succeed, but some games might not let the player pass turns.
    fn pass(&mut self) -> CommandResult;
}
