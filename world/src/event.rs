/// Immediate events emitted by game events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Msg(String), // Text output to console
}
