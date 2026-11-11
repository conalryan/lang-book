"""A scripted scenario with a fake clock, so output is identical on every run and in every port."""

from collections.abc import Callable
from datetime import UTC, datetime

from library.logic import (
    InMemoryRepository,
    LibraryService,
    ManualClock,
    describe_error,
    format_date,
    format_money,
)
from library.models import Book, Dvd, Err, LibraryItem, Loan, Member, Ok


def run_demo(write: Callable[[str], None]) -> None:
    """Run the scenario, writing each line through `write`."""
    items = InMemoryRepository[LibraryItem]()
    items.add(Book("b1", "Dune", "Frank Herbert"))
    items.add(Dvd("d1", "Alien", 117))
    items.add(Book("b2", "Emma", "Jane Austen"))

    members = InMemoryRepository[Member]()
    members.add(Member("m1", "Ada"))
    members.add(Member("m2", "Grace"))

    clock = ManualClock(datetime(2024, 1, 1, tzinfo=UTC))
    library = LibraryService(items, members, InMemoryRepository[Loan](), clock)

    def check_out(item_id: str, member_id: str) -> None:
        match library.check_out(item_id, member_id):
            case Ok(checkout):
                due = format_date(checkout.loan.due_on)
                write(f"Checked out '{checkout.item.title}' to {checkout.member.name}, due {due}")
            case Err(error):
                write(f"error: {describe_error(error)}")

    def give_back(item_id: str) -> None:
        match library.return_item(item_id):
            case Ok(receipt):
                if receipt.days_late == 0:
                    write(f"Returned '{receipt.title}' on time")
                else:
                    plural = "" if receipt.days_late == 1 else "s"
                    fee = format_money(receipt.fee_cents)
                    write(
                        f"Returned '{receipt.title}' {receipt.days_late} day{plural} late, "
                        f"fee {fee}"
                    )
            case Err(error):
                write(f"error: {describe_error(error)}")

    check_out("b1", "m1")
    check_out("d1", "m1")
    check_out("b2", "m1")
    check_out("b1", "m2")
    clock.advance_days(7)
    give_back("d1")
    clock.advance_days(10)
    give_back("b1")
    give_back("b1")
