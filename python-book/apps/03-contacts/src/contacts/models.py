"""Contact types and help text."""

from collections.abc import Mapping
from dataclasses import dataclass


@dataclass(frozen=True)
class Contact:
    """One entry. `frozen=True` makes instances immutable and hashable."""

    name: str
    phone: str


# Keyed by lower-cased name so lookups are case-insensitive. Treated as immutable.
type ContactBook = Mapping[str, Contact]

# Every command returns (new_book, text_to_print): a plain tuple.
type CommandResult = tuple[ContactBook, str]

HELP = "\n".join(
    [
        "commands:",
        "  add <name> <phone>",
        "  remove <name>",
        "  find <text>",
        "  list",
        "  groups",
        "  count",
        "  help",
        "  quit",
    ]
)
