"""Domain types: items (inheritance), repositories (generic protocol), results and errors."""

from abc import ABC, abstractmethod
from collections.abc import Callable
from dataclasses import dataclass
from datetime import datetime
from typing import Protocol


@dataclass(frozen=True)
class Ok[T]:
    """Success carrying a value."""

    value: T


@dataclass(frozen=True)
class Err[E]:
    """Failure carrying an error."""

    error: E


type Result[T, E] = Ok[T] | Err[E]


class Entity(Protocol):
    """Anything stored in a repository needs a stable id (structural: no inheritance needed)."""

    @property
    def id(self) -> str:
        """Unique identifier."""
        ...


class Repository[T: Entity](Protocol):
    """Generic protocol: `T` must satisfy `Entity` (a bound, like Rust's `T: Entity`)."""

    def get(self, id: str) -> T | None:
        """Look up by id."""
        ...

    def add(self, item: T) -> None:
        """Insert or replace."""
        ...

    def remove(self, id: str) -> bool:
        """Delete; True if something was removed."""
        ...

    def all(self) -> list[T]:
        """Every item."""
        ...

    def find(self, predicate: Callable[[T], bool]) -> list[T]:
        """Items matching `predicate`."""
        ...


class LibraryItem(ABC):
    """Inheritance: Book and Dvd *are* LibraryItems and share id/title.

    The abstract members force every subclass to define its own loan period.
    """

    def __init__(self, id: str, title: str) -> None:
        self.id = id
        self.title = title

    @property
    @abstractmethod
    def loan_days(self) -> int:
        """How long this kind of item may be borrowed."""

    @abstractmethod
    def describe(self) -> str:
        """Human readable description."""


class Book(LibraryItem):
    """A book: 14-day loan."""

    def __init__(self, id: str, title: str, author: str) -> None:
        super().__init__(id, title)
        self.author = author

    @property
    def loan_days(self) -> int:
        """Books are lent for 14 days."""
        return 14

    def describe(self) -> str:
        """`<title> by <author>`."""
        return f"{self.title} by {self.author}"


class Dvd(LibraryItem):
    """A DVD: 7-day loan."""

    def __init__(self, id: str, title: str, minutes: int) -> None:
        super().__init__(id, title)
        self.minutes = minutes

    @property
    def loan_days(self) -> int:
        """DVDs are lent for 7 days."""
        return 7

    def describe(self) -> str:
        """`<title> (<n> min)`."""
        return f"{self.title} ({self.minutes} min)"


@dataclass(frozen=True)
class Member:
    """Plain data: members are not polymorphic, so a dataclass is enough."""

    id: str
    name: str


@dataclass(frozen=True)
class Loan:
    """A loan is keyed by item id: one item can only be on loan once."""

    id: str
    member_id: str
    due_on: datetime


@dataclass(frozen=True)
class Policy:
    """Lending rules."""

    max_loans: int = 2
    fee_per_day_cents: int = 25


class Clock(Protocol):
    """Structural interface for time, so tests and demos control "now"."""

    def now(self) -> datetime:
        """The current time."""
        ...


@dataclass(frozen=True)
class Checkout:
    """What a successful checkout returns."""

    item: LibraryItem
    loan: Loan
    member: Member


@dataclass(frozen=True)
class Receipt:
    """What a successful return returns."""

    title: str
    days_late: int
    fee_cents: int


@dataclass(frozen=True)
class ItemNotFound:
    """Unknown item id."""

    id: str


@dataclass(frozen=True)
class MemberNotFound:
    """Unknown member id."""

    id: str


@dataclass(frozen=True)
class AlreadyOnLoan:
    """Item is already checked out."""

    title: str


@dataclass(frozen=True)
class NotOnLoan:
    """Item is not checked out."""

    title: str


@dataclass(frozen=True)
class LimitReached:
    """Member has too many loans."""

    name: str
    limit: int


type LibraryError = ItemNotFound | MemberNotFound | AlreadyOnLoan | NotOnLoan | LimitReached
