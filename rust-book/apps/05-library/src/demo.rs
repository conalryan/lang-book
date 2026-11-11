//! A scripted scenario with a fake clock, so output is identical on every run and in every port.

use crate::date::Date;
use crate::logic::{InMemoryRepository, LibraryService, ManualClock, format_money};
use crate::models::{Book, Dvd, LibraryItem, Loan, Member, Policy, Repository};

type Service<'a> = LibraryService<
    'a,
    InMemoryRepository<Box<dyn LibraryItem>>,
    InMemoryRepository<Member>,
    InMemoryRepository<Loan>,
>;

fn check_out(service: &mut Service, write: &mut impl FnMut(&str), item_id: &str, member_id: &str) {
    match service.check_out(item_id, member_id) {
        Ok(checkout) => write(&format!(
            "Checked out '{}' to {}, due {}",
            checkout.title, checkout.member_name, checkout.due_on
        )),
        Err(error) => write(&format!("error: {error}")),
    }
}

fn give_back(service: &mut Service, write: &mut impl FnMut(&str), item_id: &str) {
    match service.return_item(item_id) {
        Ok(receipt) if receipt.days_late == 0 => {
            write(&format!("Returned '{}' on time", receipt.title));
        }
        Ok(receipt) => {
            let plural = if receipt.days_late == 1 { "" } else { "s" };
            write(&format!(
                "Returned '{}' {} day{plural} late, fee {}",
                receipt.title,
                receipt.days_late,
                format_money(receipt.fee_cents)
            ));
        }
        Err(error) => write(&format!("error: {error}")),
    }
}

/// Run the scenario, writing each line through `write`.
pub fn run_demo(write: &mut impl FnMut(&str)) {
    let mut items = InMemoryRepository::<Box<dyn LibraryItem>>::new();
    // `Box<Book>` coerces to `Box<dyn LibraryItem>` where the parameter type is known.
    items.add(Box::new(Book::new("b1", "Dune", "Frank Herbert")));
    items.add(Box::new(Dvd::new("d1", "Alien", 117)));
    items.add(Box::new(Book::new("b2", "Emma", "Jane Austen")));

    let mut members = InMemoryRepository::new();
    members.add(Member::new("m1", "Ada"));
    members.add(Member::new("m2", "Grace"));

    let clock = ManualClock::new(Date::from_ymd(2024, 1, 1));
    let mut library = LibraryService::new(
        items,
        members,
        InMemoryRepository::new(),
        &clock,
        Policy::default(),
    );

    check_out(&mut library, write, "b1", "m1");
    check_out(&mut library, write, "d1", "m1");
    check_out(&mut library, write, "b2", "m1");
    check_out(&mut library, write, "b1", "m2");
    clock.advance_days(7);
    give_back(&mut library, write, "d1");
    clock.advance_days(10);
    give_back(&mut library, write, "b1");
    give_back(&mut library, write, "b1");
}
