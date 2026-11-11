"""Contact book behaviour. Every function returns new data instead of mutating its input."""

from collections.abc import Callable, Iterable
from typing import Final

from contacts.models import HELP, CommandResult, Contact, ContactBook

_NO_CONTACTS: Final = "(no contacts)"


def _key(name: str) -> str:
    return name.lower()


def _by_name(contact: Contact) -> str:
    return contact.name.casefold()


def add_contact(book: ContactBook, contact: Contact) -> ContactBook | None:
    """Return a new book with `contact` added, or None if the name is taken."""
    key = _key(contact.name)
    if key in book:
        return None
    return {**book, key: contact}  # dict unpacking copies; the original is untouched


def remove_contact(book: ContactBook, name: str) -> ContactBook | None:
    """Return a new book without `name`, or None if it is missing."""
    key = _key(name)
    if key not in book:
        return None
    return {k: v for k, v in book.items() if k != key}


def find_contacts(book: ContactBook, text: str, limit: int | None = None) -> list[Contact]:
    """Case-insensitive substring match on names, sorted. `limit=None` means no limit."""
    needle = text.lower()
    matches = sorted((c for c in book.values() if needle in c.name.lower()), key=_by_name)
    return matches if limit is None else matches[:limit]


def partition[T](items: Iterable[T], predicate: Callable[[T], bool]) -> tuple[list[T], list[T]]:
    """Split into (matching, rest) in one pass; returning a tuple avoids filtering twice."""
    yes: list[T] = []
    no: list[T] = []
    for item in items:
        (yes if predicate(item) else no).append(item)
    return yes, no


def group_by_initial(book: ContactBook) -> dict[str, list[str]]:
    """Group names by upper-cased first letter, with keys sorted."""
    groups: dict[str, list[str]] = {}
    for contact in sorted(book.values(), key=_by_name):
        initial = (contact.name[:1] or "?").upper()
        groups.setdefault(initial, []).append(contact.name)
    return dict(sorted(groups.items(), key=lambda item: item[0].casefold()))


def format_contact(contact: Contact) -> str:
    """`<name> <phone>`."""
    return f"{contact.name} {contact.phone}"


def split_command(line: str) -> tuple[str, list[str]]:
    """Return (command_word, args). Starred unpacking collects the rest into a list."""
    command, *args = line.split() or [""]
    return command.lower(), args


def execute(book: ContactBook, line: str) -> CommandResult:
    """Interpret one line. Pure: takes a book, returns a new book plus the text to print."""
    command, args = split_command(line)

    match command, args:
        case "", _:
            return book, ""
        case "help", _:
            return book, HELP
        case "add", [name, phone]:
            updated = add_contact(book, Contact(name, phone))
            if updated is None:
                return book, f"error: {name} already exists"
            return updated, f"Added {name}"
        case "add", _:
            return book, "error: usage: add <name> <phone>"
        case "remove", [name, *_]:
            updated = remove_contact(book, name)
            if updated is None:
                return book, f"error: {name} not found"
            return updated, f"Removed {name}"
        case "remove", []:
            return book, "error: usage: remove <name>"
        case "find", [text, *_]:
            matches = find_contacts(book, text)
            return book, "\n".join(map(format_contact, matches)) or "(no matches)"
        case "find", []:
            return book, "error: usage: find <text>"
        case "list", _:
            return book, "\n".join(map(format_contact, find_contacts(book, ""))) or _NO_CONTACTS
        case "groups", _:
            lines = [
                f"{initial}: {', '.join(names)}"
                for initial, names in group_by_initial(book).items()
            ]
            return book, "\n".join(lines) or _NO_CONTACTS
        case "count", _:
            return book, f"{len(book)} contact{'' if len(book) == 1 else 's'}"
        case _:
            return book, f"error: unknown command '{command}' (try 'help')"


def run_session(
    lines: Iterable[str],
    write: Callable[[str], None],
    initial: ContactBook | None = None,
) -> ContactBook:
    """Run lines until `quit` or end of input. Returns the final book."""
    book: ContactBook = {} if initial is None else initial
    for line in lines:
        if line.strip().lower() == "quit":
            write("Bye")
            break
        book, output = execute(book, line)
        if output:
            write(output)
    return book
