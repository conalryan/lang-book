use crate::logic::{
    add_contact, execute, find_contacts, group_by_initial, partition, remove_contact, run_session,
    split_command,
};
use crate::models::{Contact, ContactBook};

fn contact(name: &str, phone: &str) -> Contact {
    Contact {
        name: name.to_string(),
        phone: phone.to_string(),
    }
}

fn seeded() -> ContactBook {
    let mut book = ContactBook::new();
    for (name, phone) in [
        ("Alice", "555-0001"),
        ("adam", "555-0002"),
        ("Bob", "555-0003"),
    ] {
        add_contact(&mut book, contact(name, phone));
    }
    book
}

fn names(contacts: &[&Contact]) -> Vec<String> {
    contacts.iter().map(|c| c.name.clone()).collect()
}

#[test]
fn add_rejects_duplicates_case_insensitively() {
    let mut book = seeded();
    assert!(!add_contact(&mut book, contact("ALICE", "9")));
    assert_eq!(book.len(), 3);
}

#[test]
fn remove_and_missing() {
    let mut book = seeded();
    assert!(remove_contact(&mut book, "BOB"));
    assert!(!remove_contact(&mut book, "zed"));
    assert_eq!(book.len(), 2);
}

#[test]
fn find_matches_substrings_in_order_with_optional_limit() {
    let book = seeded();
    assert_eq!(names(&find_contacts(&book, "a", None)), ["adam", "Alice"]);
    assert_eq!(find_contacts(&book, "a", Some(1)).len(), 1);
}

#[test]
fn partition_returns_a_tuple() {
    let (evens, odds) = partition([1, 2, 3, 4, 5], |n| n % 2 == 0);
    assert_eq!(evens, [2, 4]);
    assert_eq!(odds, [1, 3, 5]);
}

#[test]
fn group_by_initial_groups_and_sorts() {
    let book = seeded();
    let groups: Vec<_> = group_by_initial(&book).into_iter().collect();
    assert_eq!(
        groups,
        [
            ("A".to_string(), vec!["adam", "Alice"]),
            ("B".to_string(), vec!["Bob"])
        ]
    );
}

#[test]
fn split_command_lowercases_and_collects_args() {
    assert_eq!(
        split_command("  ADD  Alice   555 "),
        ("add".to_string(), vec!["Alice", "555"])
    );
    assert_eq!(split_command(""), (String::new(), vec![]));
}

#[test]
fn list_is_sorted() {
    assert_eq!(
        execute(&mut seeded(), "list"),
        "adam 555-0002\nAlice 555-0001\nBob 555-0003"
    );
}

#[test]
fn execute_cases() {
    let cases = [
        ("add Zed 1", "Added Zed"),
        ("add Alice 1", "error: Alice already exists"),
        ("add Alice", "error: usage: add <name> <phone>"),
        ("remove Bob", "Removed Bob"),
        ("remove Zed", "error: Zed not found"),
        ("remove", "error: usage: remove <name>"),
        ("find bo", "Bob 555-0003"),
        ("find zzz", "(no matches)"),
        ("find", "error: usage: find <text>"),
        ("groups", "A: adam, Alice\nB: Bob"),
        ("count", "3 contacts"),
        ("", ""),
        ("dance", "error: unknown command 'dance' (try 'help')"),
    ];
    for (line, expected) in cases {
        assert_eq!(execute(&mut seeded(), line), expected, "{line:?}");
    }
}

#[test]
fn empty_book_and_help() {
    let mut empty = ContactBook::new();
    assert_eq!(execute(&mut empty, "list"), "(no contacts)");
    assert_eq!(execute(&mut empty, "groups"), "(no contacts)");
    assert_eq!(execute(&mut empty, "count"), "0 contacts");
    execute(&mut empty, "add A 1");
    assert_eq!(execute(&mut empty, "count"), "1 contact");
    assert!(execute(&mut empty, "help").contains("commands:"));
}

#[test]
fn session_runs_until_quit_and_returns_the_final_book() {
    let mut out = Vec::new();
    let book = run_session(
        ["add Alice 1", "", "list", "quit", "add Never 2"],
        &mut |line| {
            out.push(line.to_string());
        },
        ContactBook::new(),
    );
    assert_eq!(out, ["Added Alice", "Alice 1", "Bye"]);
    assert_eq!(book.len(), 1);
}
