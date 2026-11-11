//! A tiny calendar date. The standard library has no date type, so this stands in for
//! `chrono`/`time` (app 7 shows a real dependency).

use std::fmt;

/// A calendar date stored as days since 1970-01-01. `Copy` because it is just a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(i64);

impl Date {
    /// Build a date from year, month (1-12) and day (1-31) using the proleptic Gregorian calendar.
    pub fn from_ymd(year: i64, month: i64, day: i64) -> Self {
        let y = if month <= 2 { year - 1 } else { year };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        Self(era * 146_097 + doe - 719_468)
    }

    /// This date moved by `days` (negative goes back).
    #[must_use]
    pub fn add_days(self, days: i64) -> Self {
        Self(self.0 + days)
    }

    /// Whole days from `other` to `self` (negative if `self` is earlier).
    pub fn days_since(self, other: Self) -> i64 {
        self.0 - other.0
    }

    fn to_ymd(self) -> (i64, i64, i64) {
        let z = self.0 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        (year, month, day)
    }
}

impl fmt::Display for Date {
    /// `YYYY-MM-DD`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day) = self.to_ymd();
        write!(f, "{year:04}-{month:02}-{day:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::Date;

    #[test]
    fn formats_and_round_trips() {
        assert_eq!(Date::from_ymd(1970, 1, 1).to_string(), "1970-01-01");
        assert_eq!(Date::from_ymd(2024, 1, 1).to_string(), "2024-01-01");
        assert_eq!(Date::from_ymd(2024, 2, 29).to_string(), "2024-02-29");
        assert_eq!(Date::from_ymd(1969, 12, 31).to_string(), "1969-12-31");
    }

    #[test]
    fn adds_days_across_month_and_year_ends() {
        assert_eq!(
            Date::from_ymd(2024, 1, 31).add_days(1).to_string(),
            "2024-02-01"
        );
        assert_eq!(
            Date::from_ymd(2024, 12, 31).add_days(1).to_string(),
            "2025-01-01"
        );
        assert_eq!(
            Date::from_ymd(2024, 3, 1).add_days(-1).to_string(),
            "2024-02-29"
        );
    }

    #[test]
    fn counts_days_between() {
        let (a, b) = (Date::from_ymd(2024, 1, 15), Date::from_ymd(2024, 1, 18));
        assert_eq!(b.days_since(a), 3);
        assert_eq!(a.days_since(b), -3);
        assert!(a < b);
    }
}
