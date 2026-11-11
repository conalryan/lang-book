use crate::date::Date;
use crate::demo::run_demo;
use crate::logic::{InMemoryRepository, LibraryService, ManualClock, format_money};
use crate::models::{
    Book, Dvd, Entity, LibraryError, LibraryItem, Loan, Member, Policy, Receipt, Repository,
};

type Service<'a> = LibraryService<
    'a,
    InMemoryRepository<Box<dyn LibraryItem>>,
    InMemoryRepository<Member>,
    InMemoryRepository<Loan>,
>;

fn setup(clock: &ManualClock, max_loans: usize) -> Service<'_> {
    let mut items = InMemoryRepository::<Box<dyn LibraryItem>>::new();
    items.add(Box::new(Book::new("b1", "Dune", "Frank Herbert")));
    items.add(Box::new(Dvd::new("d1", "Alien", 117)));
    items.add(Box::new(Book::new("b2", "Emma", "Jane Austen")));
    let mut members = InMemoryRepository::new();
    members.add(Member::new("m1", "Ada"));
    LibraryService::new(
        items,
        members,
        InMemoryRepository::new(),
        clock,
        Policy {
            max_loans,
            fee_per_day_cents: 25,
        },
    )
}

fn start() -> ManualClock {
    ManualClock::new(Date::from_ymd(2024, 1, 1))
}

#[test]
fn item_types_share_a_trait_and_differ_in_behaviour() {
    let (book, dvd) = (
        Book::new("b", "Dune", "Frank Herbert"),
        Dvd::new("d", "Alien", 117),
    );
    assert_eq!(book.describe(), "Dune by Frank Herbert");
    assert_eq!(dvd.describe(), "Alien (117 min)");
    assert_eq!((book.loan_days(), dvd.loan_days()), (14, 7));
}

#[test]
fn trait_objects_work_through_the_supertrait() {
    let items: Vec<Box<dyn LibraryItem>> = vec![
        Box::new(Book::new("b", "Dune", "F")),
        Box::new(Dvd::new("d", "Alien", 1)),
    ];
    assert_eq!(items.iter().map(Entity::id).collect::<Vec<_>>(), ["b", "d"]);
}

#[test]
fn repository_add_find_all_remove() {
    let mut repo = InMemoryRepository::new();
    repo.add(Member::new("1", "Ada"));
    repo.add(Member::new("2", "Grace"));
    assert_eq!(repo.get("1").unwrap().name, "Ada");
    assert_eq!(
        repo.find(|m| m.name.starts_with('G')),
        [&Member::new("2", "Grace")]
    );
    assert_eq!(repo.all().len(), 2);
    assert!(repo.remove("1"));
    assert!(!repo.remove("1"));
    assert!(repo.get("1").is_none());
}

#[test]
fn adding_the_same_id_replaces() {
    let mut repo = InMemoryRepository::new();
    repo.add(Member::new("1", "Ada"));
    repo.add(Member::new("1", "Ada L."));
    assert_eq!(repo.all().len(), 1);
    assert_eq!(repo.get("1").unwrap().name, "Ada L.");
}

#[test]
fn money_formats_cents() {
    assert_eq!(format_money(75), "$0.75");
    assert_eq!(format_money(1205), "$12.05");
}

#[test]
fn checkout_uses_the_items_own_loan_period() {
    let clock = start();
    let mut library = setup(&clock, 2);
    assert_eq!(
        library.check_out("b1", "m1").unwrap().due_on.to_string(),
        "2024-01-15"
    );
    assert_eq!(
        library.check_out("d1", "m1").unwrap().due_on.to_string(),
        "2024-01-08"
    );
}

#[test]
fn checkout_unknown_ids() {
    let clock = start();
    let mut library = setup(&clock, 2);
    assert_eq!(
        library.check_out("zzz", "m1"),
        Err(LibraryError::ItemNotFound("zzz".into()))
    );
    assert_eq!(
        library.check_out("b1", "zzz"),
        Err(LibraryError::MemberNotFound("zzz".into()))
    );
}

#[test]
fn double_checkout_and_limit() {
    let clock = start();
    let mut library = setup(&clock, 1);
    library.check_out("b1", "m1").unwrap();
    assert_eq!(
        library.check_out("b1", "m1"),
        Err(LibraryError::AlreadyOnLoan("Dune".into()))
    );
    assert_eq!(
        library.check_out("b2", "m1"),
        Err(LibraryError::LimitReached {
            name: "Ada".into(),
            limit: 1
        })
    );
}

#[test]
fn late_return_charges_per_day() {
    let clock = start();
    let mut library = setup(&clock, 2);
    library.check_out("b1", "m1").unwrap();
    clock.advance_days(17);
    assert_eq!(
        library.return_item("b1"),
        Ok(Receipt {
            title: "Dune".into(),
            days_late: 3,
            fee_cents: 75
        })
    );
}

#[test]
fn on_time_return_is_free() {
    let clock = start();
    let mut library = setup(&clock, 2);
    library.check_out("d1", "m1").unwrap();
    clock.advance_days(7);
    assert_eq!(
        library.return_item("d1"),
        Ok(Receipt {
            title: "Alien".into(),
            days_late: 0,
            fee_cents: 0
        })
    );
}

#[test]
fn return_errors() {
    let clock = start();
    let mut library = setup(&clock, 2);
    assert_eq!(
        library.return_item("zzz"),
        Err(LibraryError::ItemNotFound("zzz".into()))
    );
    assert_eq!(
        library.return_item("b1"),
        Err(LibraryError::NotOnLoan("Dune".into()))
    );
}

#[test]
fn errors_describe_themselves() {
    assert_eq!(
        LibraryError::ItemNotFound("x".into()).to_string(),
        "item 'x' not found"
    );
    assert_eq!(
        LibraryError::MemberNotFound("x".into()).to_string(),
        "member 'x' not found"
    );
    assert_eq!(
        LibraryError::NotOnLoan("T".into()).to_string(),
        "'T' is not checked out"
    );
}

#[test]
fn demo_prints_the_scripted_scenario() {
    let mut lines = Vec::new();
    run_demo(&mut |line| lines.push(line.to_string()));
    assert_eq!(
        lines,
        [
            "Checked out 'Dune' to Ada, due 2024-01-15",
            "Checked out 'Alien' to Ada, due 2024-01-08",
            "error: Ada has reached the limit of 2 loans",
            "error: 'Dune' is already checked out",
            "Returned 'Alien' on time",
            "Returned 'Dune' 3 days late, fee $0.75",
            "error: 'Dune' is not checked out",
        ]
    );
}
