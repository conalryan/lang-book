"""Repository implementation, clock and the library service."""

import math
from collections.abc import Callable
from datetime import datetime, timedelta

from library.models import (
    AlreadyOnLoan,
    Checkout,
    Clock,
    Entity,
    Err,
    ItemNotFound,
    LibraryError,
    LibraryItem,
    LimitReached,
    Loan,
    Member,
    MemberNotFound,
    NotOnLoan,
    Ok,
    Policy,
    Receipt,
    Repository,
    Result,
)


class InMemoryRepository[T: Entity]:
    """Generic class: one implementation serves items, members and loans.

    It never names `Repository`: matching the protocol's shape is enough (structural typing).
    """

    def __init__(self) -> None:
        self._store: dict[str, T] = {}

    def get(self, id: str) -> T | None:
        """Look up by id."""
        return self._store.get(id)

    def add(self, item: T) -> None:
        """Insert or replace."""
        self._store[item.id] = item

    def remove(self, id: str) -> bool:
        """Delete; True if something was removed."""
        return self._store.pop(id, None) is not None

    def all(self) -> list[T]:
        """Every item, in insertion order."""
        return list(self._store.values())

    def find(self, predicate: Callable[[T], bool]) -> list[T]:
        """Items matching `predicate`."""
        return [item for item in self._store.values() if predicate(item)]


class ManualClock:
    """A clock you can move forward by hand."""

    def __init__(self, start: datetime) -> None:
        self._current = start

    def now(self) -> datetime:
        """The current (fake) time."""
        return self._current

    def advance_days(self, days: int) -> None:
        """Move time forward."""
        self._current += timedelta(days=days)


def add_days(date: datetime, days: int) -> datetime:
    """`date` plus whole days."""
    return date + timedelta(days=days)


def format_date(date: datetime) -> str:
    """`YYYY-MM-DD`."""
    return date.strftime("%Y-%m-%d")


def format_money(cents: int) -> str:
    """`$0.75`."""
    return f"${cents / 100:.2f}"


def describe_error(error: LibraryError) -> str:
    """Message for each error variant."""
    match error:
        case ItemNotFound(id):
            return f"item '{id}' not found"
        case MemberNotFound(id):
            return f"member '{id}' not found"
        case AlreadyOnLoan(title):
            return f"'{title}' is already checked out"
        case NotOnLoan(title):
            return f"'{title}' is not checked out"
        case LimitReached(name, limit):
            return f"{name} has reached the limit of {limit} loans"


class LibraryService:
    """Composition: the service *has* repositories, a clock and a policy; it inherits from none.

    Dependencies are injected through the constructor, which keeps it testable.
    """

    def __init__(
        self,
        items: Repository[LibraryItem],
        members: Repository[Member],
        loans: Repository[Loan],
        clock: Clock,
        policy: Policy | None = None,
    ) -> None:
        self._items = items
        self._members = members
        self._loans = loans
        self._clock = clock
        self._policy = policy or Policy()

    def check_out(self, item_id: str, member_id: str) -> Result[Checkout, LibraryError]:
        """Lend an item to a member if the rules allow it."""
        if (item := self._items.get(item_id)) is None:
            return Err(ItemNotFound(item_id))
        if (member := self._members.get(member_id)) is None:
            return Err(MemberNotFound(member_id))
        if self._loans.get(item_id) is not None:
            return Err(AlreadyOnLoan(item.title))
        if (
            len(self._loans.find(lambda loan: loan.member_id == member_id))
            >= self._policy.max_loans
        ):
            return Err(LimitReached(member.name, self._policy.max_loans))

        # Polymorphism: loan_days comes from whichever subclass `item` really is.
        loan = Loan(item_id, member_id, add_days(self._clock.now(), item.loan_days))
        self._loans.add(loan)
        return Ok(Checkout(item, loan, member))

    def return_item(self, item_id: str) -> Result[Receipt, LibraryError]:
        """Take an item back and compute any late fee."""
        if (item := self._items.get(item_id)) is None:
            return Err(ItemNotFound(item_id))
        if (loan := self._loans.get(item_id)) is None:
            return Err(NotOnLoan(item.title))

        overdue = (self._clock.now() - loan.due_on) / timedelta(days=1)
        days_late = max(0, math.ceil(overdue))
        self._loans.remove(item_id)
        return Ok(Receipt(item.title, days_late, days_late * self._policy.fee_per_day_cents))
