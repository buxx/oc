//! Rule based AIs able to play one side of a battle (see [`AiKind`]).
pub mod capture;
pub mod combat;
pub mod command;
pub mod defensive;
pub mod kind;
pub mod offensive;
pub mod perception;

pub use command::Command;
pub use defensive::Defensive;
pub use kind::AiKind;
pub use offensive::Offensive;
pub use perception::Perception;

/// AI playing one side: decides commands from what the side knows
pub trait Commander: Send {
    /// Commands to apply now (called regularly by the server)
    fn tick(&mut self, perception: &Perception) -> Vec<Command>;
}
