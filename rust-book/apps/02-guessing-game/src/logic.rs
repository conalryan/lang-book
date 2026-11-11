//! Game rules, kept free of I/O so tests can pass plain arrays.

use std::cmp::Ordering;

use crate::models::{GameConfig, GameOutcome, Hint, PlayerInput};

/// Deterministic 32-bit PRNG (mulberry32). Same seed gives the same sequence in any language.
///
/// Rust: the returned closure *owns* `state` (`move`) and mutates it, hence `FnMut`.
pub fn create_rng(seed: u32) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || {
        state = state.wrapping_add(0x6D2B_79F5);
        let mut t = state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 0x3D));
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

/// Pick an integer in `[min, max]` inclusive.
pub fn pick_secret(rng: &mut impl FnMut() -> f64, config: &GameConfig) -> u32 {
    let span = f64::from(config.max - config.min + 1);
    config.min + (rng() * span) as u32 // `as` truncates toward zero, like Math.floor here
}

/// Compare a guess with the secret. `Ordering` is the standard three-way result.
pub fn judge(guess: u32, secret: u32) -> Hint {
    match guess.cmp(&secret) {
        Ordering::Less => Hint::Low,
        Ordering::Greater => Hint::High,
        Ordering::Equal => Hint::Correct,
    }
}

/// Text shown for a hint. `match` must be exhaustive, so adding a variant breaks the build here.
pub fn describe_hint(hint: Hint) -> &'static str {
    match hint {
        Hint::Low => "Too low",
        Hint::High => "Too high",
        Hint::Correct => "Correct!",
    }
}

/// `q`/`quit` ends the game; digits in range are guesses; everything else is invalid.
pub fn parse_input(line: &str, config: &GameConfig) -> PlayerInput {
    let text = line.trim().to_lowercase();
    if text == "q" || text == "quit" {
        return PlayerInput::Quit;
    }
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return PlayerInput::Invalid;
    }
    // Very long digit strings overflow u32; they are out of range anyway.
    match text.parse::<u32>() {
        Ok(value) if (config.min..=config.max).contains(&value) => PlayerInput::Guess(value),
        _ => PlayerInput::Invalid,
    }
}

/// Opening message.
pub fn intro(config: &GameConfig) -> String {
    format!(
        "Guess a number between {} and {}. You have {} attempts. Type 'q' to quit.",
        config.min, config.max, config.max_attempts
    )
}

/// Drive one game. Invalid input does not use up an attempt.
///
/// `lines` can be anything iterable over string-likes: an array in tests, stdin in the CLI.
#[allow(
    clippy::needless_continue,
    reason = "explicit `continue` is the point of this app"
)]
pub fn play_game<I>(
    secret: u32,
    config: &GameConfig,
    lines: I,
    write: &mut impl FnMut(&str),
) -> GameOutcome
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut attempts = 0;

    for line in lines {
        match parse_input(line.as_ref(), config) {
            PlayerInput::Quit => {
                write("Bye");
                return GameOutcome::Quit;
            }
            PlayerInput::Invalid => {
                write(&format!(
                    "Please enter a whole number between {} and {}",
                    config.min, config.max
                ));
                continue;
            }
            PlayerInput::Guess(value) => {
                attempts += 1;
                let hint = judge(value, secret);
                if hint == Hint::Correct {
                    let plural = if attempts == 1 { "" } else { "s" };
                    write(&format!("Correct! You won in {attempts} attempt{plural}"));
                    return GameOutcome::Won { attempts };
                }
                write(describe_hint(hint));
                if attempts >= config.max_attempts {
                    break;
                }
            }
        }
    }

    // Reached by running out of attempts, or by input ending before the game did.
    if attempts >= config.max_attempts {
        write(&format!("Out of attempts. The number was {secret}"));
        return GameOutcome::Lost { secret };
    }
    write("Bye");
    GameOutcome::Quit
}

/// Binary-search bot: a `while` loop that halves the range each round. Returns attempts used.
pub fn solve(secret: u32, config: &GameConfig) -> u32 {
    let (mut low, mut high) = (config.min, config.max);
    let mut attempts = 0;

    while low <= high {
        let guess = low + (high - low) / 2;
        attempts += 1;
        match judge(guess, secret) {
            Hint::Correct => break,
            Hint::Low => low = guess + 1,
            Hint::High => high = guess - 1,
        }
    }
    attempts
}

#[cfg(test)]
mod tests;
