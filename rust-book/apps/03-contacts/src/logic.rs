//! Contact book behaviour.
//!
//! Rust: the book is mutated in place through `&mut`, which the borrow checker guarantees is
//! exclusive. TypeScript and Python copy instead, because aliasing makes in-place edits risky.

use std::collections::BTreeMap;

use crate::models::{Contact, ContactBook, HELP};

fn key_of(name: &str) -> String {
    name.to_lowercase()
}

/// Add `contact`. Returns `false` (and changes nothing) if the name is taken.
pub fn add_contact(book: &mut ContactBook, contact: Contact) -> bool {
    use std::collections::btree_map::Entry;
    match book.entry(key_of(&contact.name)) {
        Entry::Occupied(_) => false,
        Entry::Vacant(slot) => {
            slot.insert(contact);
            true
        }
    }
}

/// Remove by name. Returns `false` if it was not there.
pub fn remove_contact(book: &mut ContactBook, name: &str) -> bool {
    book.remove(&key_of(name)).is_some()
}

/// Case-insensitive substring match on names, in name order. `limit: None` means no limit.
///
/// The returned references borrow from `book`; the compiler ties their lifetime to it.
pub fn find_contacts<'a>(
    book: &'a ContactBook,
    text: &str,
    limit: Option<usize>,
) -> Vec<&'a Contact> {
    let needle = text.to_lowercase();
    book.iter()
        .filter(|(key, _)| key.contains(&needle))
        .map(|(_, contact)| contact)
        .take(limit.unwrap_or(usize::MAX))
        .collect()
}

/// Split into `(matching, rest)` in one pass; returning a tuple avoids filtering twice.
pub fn partition<T>(
    items: impl IntoIterator<Item = T>,
    predicate: impl Fn(&T) -> bool,
) -> (Vec<T>, Vec<T>) {
    let mut yes = Vec::new();
    let mut no = Vec::new();
    for item in items {
        if predicate(&item) {
            yes.push(item)
        } else {
            no.push(item)
        }
    }
    (yes, no)
}

/// Group names by upper-cased first letter, with keys sorted.
pub fn group_by_initial(book: &ContactBook) -> BTreeMap<String, Vec<&str>> {
    let mut groups: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for contact in book.values() {
        let initial = contact
            .name
            .chars()
            .next()
            .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string());
        groups.entry(initial).or_default().push(&contact.name);
    }
    groups
}

/// `<name> <phone>`.
pub fn format_contact(contact: &Contact) -> String {
    format!("{} {}", contact.name, contact.phone)
}

/// Return `(command_word, args)`. The `&str` args borrow from `line`: no copying.
pub fn split_command(line: &str) -> (String, Vec<&str>) {
    let mut words = line.split_whitespace();
    let command = words.next().unwrap_or("").to_lowercase();
    (command, words.collect())
}

fn lines_or(empty: &str, lines: impl Iterator<Item = String>) -> String {
    let joined = lines.collect::<Vec<_>>().join("\n");
    if joined.is_empty() {
        empty.to_string()
    } else {
        joined
    }
}

/// Interpret one line, mutating `book`, and return the text to print.
pub fn execute(book: &mut ContactBook, line: &str) -> String {
    let (command, args) = split_command(line);

    // Matching on a tuple of (&str, slice) lets one `match` handle command and arity.
    match (command.as_str(), args.as_slice()) {
        ("", _) => String::new(),
        ("help", _) => HELP.to_string(),
        ("add", [name, phone]) => {
            let contact = Contact {
                name: (*name).to_string(),
                phone: (*phone).to_string(),
            };
            if add_contact(book, contact) {
                format!("Added {name}")
            } else {
                format!("error: {name} already exists")
            }
        }
        ("add", _) => "error: usage: add <name> <phone>".to_string(),
        ("remove", [name, ..]) => {
            if remove_contact(book, name) {
                format!("Removed {name}")
            } else {
                format!("error: {name} not found")
            }
        }
        ("remove", []) => "error: usage: remove <name>".to_string(),
        ("find", [text, ..]) => {
            let matches = find_contacts(book, text, None);
            if matches.is_empty() {
                "(no matches)".to_string()
            } else {
                matches
                    .into_iter()
                    .map(format_contact)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        ("find", []) => "error: usage: find <text>".to_string(),
        ("list", _) => lines_or(
            "(no contacts)",
            find_contacts(book, "", None)
                .into_iter()
                .map(format_contact),
        ),
        ("groups", _) => lines_or(
            "(no contacts)",
            group_by_initial(book)
                .into_iter()
                .map(|(initial, names)| format!("{initial}: {}", names.join(", "))),
        ),
        ("count", _) => {
            let count = book.len();
            format!("{count} contact{}", if count == 1 { "" } else { "s" })
        }
        (other, _) => format!("error: unknown command '{other}' (try 'help')"),
    }
}

/// Run lines until `quit` or end of input. Takes ownership of `initial` and returns the final book.
pub fn run_session<I>(lines: I, write: &mut impl FnMut(&str), initial: ContactBook) -> ContactBook
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut book = initial;
    for line in lines {
        let line = line.as_ref();
        if line.trim().eq_ignore_ascii_case("quit") {
            write("Bye");
            break;
        }
        let output = execute(&mut book, line);
        if !output.is_empty() {
            write(&output);
        }
    }
    book
}

#[cfg(test)]
mod tests;
