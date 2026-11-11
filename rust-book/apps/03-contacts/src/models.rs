//! Contact types and help text.

use std::collections::BTreeMap;

/// One entry. `String` fields mean the struct owns its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    pub name: String,
    pub phone: String,
}

/// Keyed by lower-cased name so lookups are case-insensitive.
///
/// Rust: a `BTreeMap` iterates in key order, so "sorted by name" comes for free.
pub type ContactBook = BTreeMap<String, Contact>;

/// Shown by the `help` command.
pub const HELP: &str = "commands:
  add <name> <phone>
  remove <name>
  find <text>
  list
  groups
  count
  help
  quit";
