from datetime import UTC, datetime

import pytest
from library.demo import run_demo
from library.logic import (
    InMemoryRepository,
    LibraryService,
    ManualClock,
    add_days,
    describe_error,
    format_date,
    format_money,
)
from library.models import (
    AlreadyOnLoan,
    Book,
    Dvd,
    Err,
    ItemNotFound,
    LibraryItem,
    LimitReached,
    Loan,
    Member,
    MemberNotFound,
    NotOnLoan,
    Ok,
    Policy,
    Receipt,
)


def setup(max_loans: int = 2) -> tuple[LibraryService, ManualClock]:
    items = InMemoryRepository[LibraryItem]()
    items.add(Book("b1", "Dune", "Frank Herbert"))
    items.add(Dvd("d1", "Alien", 117))
    items.add(Book("b2", "Emma", "Jane Austen"))
    members = InMemoryRepository[Member]()
    members.add(Member("m1", "Ada"))
    clock = ManualClock(datetime(2024, 1, 1, tzinfo=UTC))
    service = LibraryService(items, members, InMemoryRepository[Loan](), clock, Policy(max_loans))
    return service, clock


def test_item_hierarchy_shares_fields_and_overrides_behaviour() -> None:
    book, dvd = Book("b", "Dune", "Frank Herbert"), Dvd("d", "Alien", 117)
    assert book.describe() == "Dune by Frank Herbert"
    assert dvd.describe() == "Alien (117 min)"
    assert (book.loan_days, dvd.loan_days) == (14, 7)


def test_abstract_base_cannot_be_instantiated() -> None:
    with pytest.raises(TypeError):
        LibraryItem("x", "y")  # type: ignore[abstract]


def test_repository_add_find_all_remove() -> None:
    repo = InMemoryRepository[Member]()
    repo.add(Member("1", "Ada"))
    repo.add(Member("2", "Grace"))
    found = repo.get("1")
    assert found is not None
    assert found.name == "Ada"
    assert repo.find(lambda m: m.name.startswith("G")) == [Member("2", "Grace")]
    assert len(repo.all()) == 2
    assert repo.remove("1") is True
    assert repo.remove("1") is False
    assert repo.get("1") is None


def test_date_and_money_helpers() -> None:
    assert format_date(add_days(datetime(2024, 1, 31, tzinfo=UTC), 1)) == "2024-02-01"
    assert format_money(75) == "$0.75"


def test_checkout_uses_the_items_own_loan_period() -> None:
    service, _ = setup()
    book, dvd = service.check_out("b1", "m1"), service.check_out("d1", "m1")
    assert isinstance(book, Ok)
    assert isinstance(dvd, Ok)
    assert format_date(book.value.loan.due_on) == "2024-01-15"
    assert format_date(dvd.value.loan.due_on) == "2024-01-08"


@pytest.mark.parametrize(
    ("item_id", "member_id", "error"),
    [
        ("zzz", "m1", ItemNotFound("zzz")),
        ("b1", "zzz", MemberNotFound("zzz")),
    ],
)
def test_checkout_unknown_ids(item_id: str, member_id: str, error: object) -> None:
    assert setup()[0].check_out(item_id, member_id) == Err(error)


def test_double_checkout_and_limit() -> None:
    service, _ = setup(max_loans=1)
    service.check_out("b1", "m1")
    assert service.check_out("b1", "m1") == Err(AlreadyOnLoan("Dune"))
    assert service.check_out("b2", "m1") == Err(LimitReached("Ada", 1))


def test_late_return_charges_per_day() -> None:
    service, clock = setup()
    service.check_out("b1", "m1")
    clock.advance_days(17)
    assert service.return_item("b1") == Ok(Receipt("Dune", 3, 75))


def test_on_time_return_is_free() -> None:
    service, clock = setup()
    service.check_out("d1", "m1")
    clock.advance_days(7)
    assert service.return_item("d1") == Ok(Receipt("Alien", 0, 0))


def test_return_errors() -> None:
    service, _ = setup()
    assert service.return_item("zzz") == Err(ItemNotFound("zzz"))
    assert service.return_item("b1") == Err(NotOnLoan("Dune"))


def test_describe_error_covers_every_variant() -> None:
    assert describe_error(ItemNotFound("x")) == "item 'x' not found"
    assert describe_error(MemberNotFound("x")) == "member 'x' not found"
    assert describe_error(NotOnLoan("T")) == "'T' is not checked out"


def test_demo_prints_the_scripted_scenario() -> None:
    lines: list[str] = []
    run_demo(lines.append)
    assert lines == [
        "Checked out 'Dune' to Ada, due 2024-01-15",
        "Checked out 'Alien' to Ada, due 2024-01-08",
        "error: Ada has reached the limit of 2 loans",
        "error: 'Dune' is already checked out",
        "Returned 'Alien' on time",
        "Returned 'Dune' 3 days late, fee $0.75",
        "error: 'Dune' is not checked out",
    ]
