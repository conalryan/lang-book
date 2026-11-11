//! Domain types: items (trait objects), repositories (generic trait), errors.

use std::fmt;

use crate::date::Date;

/// Anything stored in a repository needs a stable id.
pub trait Entity {
    fn id(&self) -> &str;
}

/// Let a `Box<dyn Trait>` be an `Entity` too, so boxed items can live in a repository.
impl<T: Entity + ?Sized> Entity for Box<T> {
    fn id(&self) -> &str {
        (**self).id()
    }
}

/// Generic trait with a bound: `T` must be an `Entity`.
pub trait Repository<T: Entity> {
    fn get(&self, id: &str) -> Option<&T>;
    /// Insert, replacing any item with the same id.
    fn add(&mut self, item: T);
    /// Delete; `true` if something was removed.
    fn remove(&mut self, id: &str) -> bool;
    fn all(&self) -> Vec<&T>;
    fn find(&self, predicate: impl Fn(&T) -> bool) -> Vec<&T>;
}

/// Rust has no inheritance: `Book` and `Dvd` are separate structs that implement a shared trait.
/// `Entity` is a *supertrait*, so every `LibraryItem` is also an `Entity`.
pub trait LibraryItem: Entity {
    fn title(&self) -> &str;
    /// How long this kind of item may be borrowed.
    fn loan_days(&self) -> u32;
    /// Human readable description.
    fn describe(&self) -> String;
}

/// A book: 14-day loan.
#[derive(Debug, Clone)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
}

impl Book {
    pub fn new(id: &str, title: &str, author: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            author: author.into(),
        }
    }
}

impl Entity for Book {
    fn id(&self) -> &str {
        &self.id
    }
}

impl LibraryItem for Book {
    fn title(&self) -> &str {
        &self.title
    }

    fn loan_days(&self) -> u32 {
        14
    }

    fn describe(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

/// A DVD: 7-day loan.
#[derive(Debug, Clone)]
pub struct Dvd {
    pub id: String,
    pub title: String,
    pub minutes: u32,
}

impl Dvd {
    pub fn new(id: &str, title: &str, minutes: u32) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            minutes,
        }
    }
}

impl Entity for Dvd {
    fn id(&self) -> &str {
        &self.id
    }
}

impl LibraryItem for Dvd {
    fn title(&self) -> &str {
        &self.title
    }

    fn loan_days(&self) -> u32 {
        7
    }

    fn describe(&self) -> String {
        format!("{} ({} min)", self.title, self.minutes)
    }
}

/// Plain data: members are not polymorphic, so a struct is enough.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub id: String,
    pub name: String,
}

impl Member {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

impl Entity for Member {
    fn id(&self) -> &str {
        &self.id
    }
}

/// A loan is keyed by item id: one item can only be on loan once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loan {
    pub id: String,
    pub member_id: String,
    pub due_on: Date,
}

impl Entity for Loan {
    fn id(&self) -> &str {
        &self.id
    }
}

/// Lending rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    pub max_loans: usize,
    pub fee_per_day_cents: u32,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            max_loans: 2,
            fee_per_day_cents: 25,
        }
    }
}

/// A source of "today", so tests and demos control time.
pub trait Clock {
    fn today(&self) -> Date;
}

/// What a successful checkout returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkout {
    pub title: String,
    pub member_name: String,
    pub due_on: Date,
}

/// What a successful return returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub title: String,
    pub days_late: u32,
    pub fee_cents: u32,
}

/// Everything that can go wrong. Variants carry the data needed to explain the failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryError {
    ItemNotFound(String),
    MemberNotFound(String),
    AlreadyOnLoan(String),
    NotOnLoan(String),
    LimitReached { name: String, limit: usize },
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemNotFound(id) => write!(f, "item '{id}' not found"),
            Self::MemberNotFound(id) => write!(f, "member '{id}' not found"),
            Self::AlreadyOnLoan(title) => write!(f, "'{title}' is already checked out"),
            Self::NotOnLoan(title) => write!(f, "'{title}' is not checked out"),
            Self::LimitReached { name, limit } => {
                write!(f, "{name} has reached the limit of {limit} loans")
            }
        }
    }
}

impl std::error::Error for LibraryError {}
