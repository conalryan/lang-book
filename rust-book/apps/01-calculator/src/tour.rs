//! A guided tour of Rust's primitive types, variables and casts.

use std::fmt::Display;

/// Return the tour as lines so it can be tested.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::unnecessary_literal_unwrap,
    reason = "the tour demonstrates exactly these conversions"
)]
pub fn tour() -> Vec<String> {
    let mut lines = Vec::new();
    let mut say = |label: &str, value: &dyn Display| lines.push(format!("{label:<18}{value}"));

    // Bindings are immutable unless declared `mut`; a later `let` may shadow (and change the type).
    let answer = 42;
    let mut counter = 0;
    counter += 1; // compound assignment: += -= *= /= %= and the bit-ops
    let spaces = "   ";
    let spaces = spaces.len(); // shadowing: a new binding, here str -> usize

    // Integers have fixed widths and no implicit conversion; the default is i32.
    say("i32", &answer);
    say("spaces", &spaces);
    say("i32::MAX", &i32::MAX);
    say("255u8 wrapping +1", &255u8.wrapping_add(1));
    say("255u8 checked +1", &format!("{:?}", 255u8.checked_add(1))); // overflow is explicit
    say("2u128.pow(64)", &2u128.pow(64));
    say("1_000_000", &1_000_000); // underscores are purely visual

    // `/` on integers truncates toward zero and `%` keeps the dividend's sign.
    say("7 / 2", &(7 / 2));
    say("-7 / 2", &(-7 / 2));
    say("-7 % 3", &(-7 % 3));
    say("rem_euclid", &(-7i32).rem_euclid(3));
    say("7.0 / 2.0", &(7.0 / 2.0));

    // f64 is IEEE-754, same as JavaScript and Python floats.
    say("0.1 + 0.2", &(0.1 + 0.2));
    let (a, b) = (f64::NAN, f64::NAN);
    say("nan == nan", &(a == b));
    say("is_nan", &a.is_nan());

    // `as` casts never fail: they truncate, wrap or saturate.
    say("-1i32 as u32", &(-1i32 as u32));
    say("300i32 as u8", &(300i32 as u8));
    say("3.99 as i32", &(3.99f64 as i32));
    say("NAN as i32", &(a as i32));

    // str is borrowed UTF-8 text, String is owned and growable; len() counts bytes.
    let word = "héllo";
    say("len (bytes)", &word.len());
    say("chars().count()", &word.chars().count());
    say("format!", &format!("Hello, {word}! counter={counter}"));

    // bool, char (a Unicode scalar), tuples and the unit type.
    say("bool", &(answer > 10 && answer % 2 == 0));
    say("char", &'é');
    let (x, y) = (1, "two"); // tuple destructuring
    say("tuple", &format!("{x} {y}"));
    say("unit", &format!("{:?}", ()));

    // There is no null: absence is `Option`.
    let missing: Option<i32> = None;
    say("Option", &format!("{:?} {:?}", Some(5), missing));
    say("unwrap_or", &missing.unwrap_or(5));

    lines
}

#[cfg(test)]
mod tests {
    use super::tour;

    #[test]
    fn documents_rust_semantics() {
        let output = tour().join("\n");
        assert!(output.contains("0.30000000000000004"));
        assert!(output.contains("18446744073709551616"));
        assert!(output.contains("255u8 wrapping +1 0"));
        assert!(output.contains("-7 / 2            -3"));
        assert!(output.contains("-1i32 as u32      4294967295"));
        assert!(output.contains("len (bytes)       6"));
    }
}
