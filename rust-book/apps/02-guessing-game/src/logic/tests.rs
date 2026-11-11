use crate::logic::{
    create_rng, describe_hint, intro, judge, parse_input, pick_secret, play_game, solve,
};
use crate::models::{DEFAULT_CONFIG, GameConfig, GameOutcome, Hint, PlayerInput};

const CONFIG: GameConfig = GameConfig {
    min: 1,
    max: 100,
    max_attempts: 3,
};

fn play(secret: u32, inputs: &[&str]) -> (GameOutcome, Vec<String>) {
    let mut output = Vec::new();
    let outcome = play_game(secret, &CONFIG, inputs, &mut |line| {
        output.push(line.to_string())
    });
    (outcome, output)
}

#[test]
fn rng_is_deterministic() {
    let (mut a, mut b) = (create_rng(42), create_rng(42));
    assert_eq!([a(), a(), a()], [b(), b(), b()]);
}

#[test]
fn pick_secret_stays_in_range() {
    let mut rng = create_rng(7);
    assert!(
        (0..1000)
            .map(|_| pick_secret(&mut rng, &DEFAULT_CONFIG))
            .all(|n| (1..=100).contains(&n))
    );
}

#[test]
fn matches_cross_language_reference_value() {
    assert_eq!(pick_secret(&mut create_rng(1), &DEFAULT_CONFIG), 63);
}

#[test]
fn judge_and_describe_hint() {
    let hints = [judge(1, 5), judge(9, 5), judge(5, 5)];
    assert_eq!(hints, [Hint::Low, Hint::High, Hint::Correct]);
    assert_eq!(
        hints.map(describe_hint),
        ["Too low", "Too high", "Correct!"]
    );
}

#[test]
fn parse_input_cases() {
    let cases = [
        ("50", PlayerInput::Guess(50)),
        (" 7 ", PlayerInput::Guess(7)),
        ("Q", PlayerInput::Quit),
        ("quit", PlayerInput::Quit),
        ("0", PlayerInput::Invalid),
        ("101", PlayerInput::Invalid),
        ("-3", PlayerInput::Invalid),
        ("abc", PlayerInput::Invalid),
        ("", PlayerInput::Invalid),
        ("\u{ff15}\u{ff10}", PlayerInput::Invalid), // full-width digits are not ASCII digits
        ("99999999999999999999", PlayerInput::Invalid),
    ];
    for (line, expected) in cases {
        assert_eq!(parse_input(line, &CONFIG), expected, "{line:?}");
    }
}

#[test]
fn win_counts_attempts() {
    let (outcome, output) = play(30, &["10", "50", "30"]);
    assert_eq!(outcome, GameOutcome::Won { attempts: 3 });
    assert_eq!(
        output,
        ["Too low", "Too high", "Correct! You won in 3 attempts"]
    );
}

#[test]
fn singular_attempt() {
    assert_eq!(play(30, &["30"]).1, ["Correct! You won in 1 attempt"]);
}

#[test]
fn invalid_input_is_free() {
    let (outcome, output) = play(30, &["abc", "999", "30"]);
    assert_eq!(outcome, GameOutcome::Won { attempts: 1 });
    assert_eq!(
        output[..2],
        ["Please enter a whole number between 1 and 100"; 2]
    );
}

#[test]
fn loses_after_max_attempts_and_ignores_remaining_input() {
    let (outcome, output) = play(30, &["1", "2", "3", "30"]);
    assert_eq!(outcome, GameOutcome::Lost { secret: 30 });
    assert_eq!(output.last().unwrap(), "Out of attempts. The number was 30");
}

#[test]
fn quits_on_request_and_when_input_ends() {
    assert_eq!(play(30, &["q"]).0, GameOutcome::Quit);
    assert_eq!(play(30, &["5"]).0, GameOutcome::Quit);
}

#[test]
fn accepts_owned_strings_too() {
    let mut output = Vec::new();
    play_game(30, &CONFIG, vec![String::from("30")], &mut |l| {
        output.push(l.to_string())
    });
    assert_eq!(output, ["Correct! You won in 1 attempt"]);
}

#[test]
fn solve_needs_at_most_seven_attempts() {
    assert!((1..=100).all(|secret| solve(secret, &DEFAULT_CONFIG) <= 7));
}

#[test]
fn intro_describes_the_rules() {
    assert_eq!(
        intro(&DEFAULT_CONFIG),
        "Guess a number between 1 and 100. You have 10 attempts. Type 'q' to quit."
    );
}
