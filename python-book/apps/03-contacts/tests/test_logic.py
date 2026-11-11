import pytest
from contacts.logic import (
    add_contact,
    execute,
    find_contacts,
    group_by_initial,
    partition,
    remove_contact,
    run_session,
    split_command,
)
from contacts.models import Contact, ContactBook

EMPTY: ContactBook = {}


def seeded() -> ContactBook:
    book: ContactBook = EMPTY
    for name, phone in [("Alice", "555-0001"), ("adam", "555-0002"), ("Bob", "555-0003")]:
        book = add_contact(book, Contact(name, phone)) or book
    return book


def test_add_does_not_mutate_the_original() -> None:
    updated = add_contact(EMPTY, Contact("Alice", "1"))
    assert len(EMPTY) == 0
    assert updated is not None
    assert len(updated) == 1


def test_add_rejects_duplicates_case_insensitively() -> None:
    assert add_contact(seeded(), Contact("ALICE", "9")) is None


def test_remove_and_missing() -> None:
    removed = remove_contact(seeded(), "BOB")
    assert removed is not None
    assert len(removed) == 2
    assert remove_contact(seeded(), "zed") is None


def test_find_matches_substrings_sorted_with_optional_limit() -> None:
    assert [c.name for c in find_contacts(seeded(), "a")] == ["adam", "Alice"]
    assert len(find_contacts(seeded(), "a", limit=1)) == 1


def test_partition_returns_a_tuple() -> None:
    evens, odds = partition([1, 2, 3, 4, 5], lambda n: n % 2 == 0)
    assert evens == [2, 4]
    assert odds == [1, 3, 5]


def test_group_by_initial() -> None:
    assert list(group_by_initial(seeded()).items()) == [("A", ["adam", "Alice"]), ("B", ["Bob"])]


def test_split_command() -> None:
    assert split_command("  ADD  Alice   555 ") == ("add", ["Alice", "555"])
    assert split_command("") == ("", [])


def test_list_is_sorted() -> None:
    assert execute(seeded(), "list")[1] == "adam 555-0002\nAlice 555-0001\nBob 555-0003"


@pytest.mark.parametrize(
    ("line", "expected"),
    [
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
    ],
)
def test_execute(line: str, expected: str) -> None:
    assert execute(seeded(), line)[1] == expected


def test_empty_book() -> None:
    assert execute(EMPTY, "list")[1] == "(no contacts)"
    assert execute(EMPTY, "groups")[1] == "(no contacts)"
    assert execute(EMPTY, "count")[1] == "0 contacts"
    assert execute(add_contact(EMPTY, Contact("A", "1")) or EMPTY, "count")[1] == "1 contact"


def test_help() -> None:
    assert "commands:" in execute(EMPTY, "help")[1]


def test_session_runs_until_quit_and_returns_final_book() -> None:
    out: list[str] = []
    book = run_session(["add Alice 1\n", "\n", "list\n", "quit\n", "add Never 2\n"], out.append)
    assert out == ["Added Alice", "Alice 1", "Bye"]
    assert len(book) == 1
