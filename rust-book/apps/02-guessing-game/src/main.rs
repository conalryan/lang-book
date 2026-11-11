use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use guessing_game::io::{print_line, read_lines};
use guessing_game::logic::{create_rng, intro, pick_secret, play_game, solve};
use guessing_game::models::DEFAULT_CONFIG;

/// `GUESS_SEED` makes the secret reproducible in every language port.
fn seed_from_env() -> Result<u32, String> {
    if let Ok(text) = std::env::var("GUESS_SEED") {
        // Truncating to 32 bits matches the other ports.
        return text
            .trim()
            .parse::<i64>()
            .map(|seed| seed as u32)
            .map_err(|_| format!("invalid GUESS_SEED '{text}'"));
    }
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    Ok(millis as u32)
}

fn main() -> ExitCode {
    let seed = match seed_from_env() {
        Ok(seed) => seed,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };
    let secret = pick_secret(&mut create_rng(seed), &DEFAULT_CONFIG);

    if std::env::args().any(|arg| arg == "--auto") {
        let attempts = solve(secret, &DEFAULT_CONFIG);
        print_line(&format!("Bot found {secret} in {attempts} attempts"));
    } else {
        print_line(&intro(&DEFAULT_CONFIG));
        play_game(secret, &DEFAULT_CONFIG, read_lines(), &mut |line| {
            print_line(line)
        });
    }
    ExitCode::SUCCESS
}
