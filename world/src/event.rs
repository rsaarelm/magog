use calx_ecs::Entity;

/// Immediate events emitted by game events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    /// Text output to console.
    ///
    /// This is the fallback event type, try to do things with more immediate animations.
    Msg(String),

    /// Damage dealt to an entity. Use negative values to show healing.
    Damage { entity: Entity, amount: i32 }
}
