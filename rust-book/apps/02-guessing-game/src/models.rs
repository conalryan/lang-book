//! Types for the guessing game.

/// Feedback for a single guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    Low,
    High,
    Correct,
}

/// Range and attempt limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    pub min: u32,
    pub max: u32,
    pub max_attempts: u32,
}

/// 1..=100 with 10 attempts.
pub const DEFAULT_CONFIG: GameConfig = GameConfig {
    min: 1,
    max: 100,
    max_attempts: 10,
};

/// What the player typed, after parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerInput {
    /// A valid in-range guess.
    Guess(u32),
    /// `q` or `quit`.
    Quit,
    /// Anything else.
    Invalid,
}

/// How a game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Won { attempts: u32 },
    Lost { secret: u32 },
    Quit,
}
