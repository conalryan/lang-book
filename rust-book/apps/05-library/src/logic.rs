//! Repository implementation, clock and the library service.

use std::cell::Cell;

use crate::date::Date;
use crate::models::{
    Checkout, Clock, Entity, LibraryError, LibraryItem, Loan, Member, Policy, Receipt, Repository,
};

/// Generic struct: one implementation serves items, members and loans.
#[derive(Debug)]
pub struct InMemoryRepository<T> {
    items: Vec<T>,
}

impl<T> Default for InMemoryRepository<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Entity> Repository<T> for InMemoryRepository<T> {
    fn get(&self, id: &str) -> Option<&T> {
        self.items.iter().find(|item| item.id() == id)
    }

    fn add(&mut self, item: T) {
        match self
            .items
            .iter()
            .position(|existing| existing.id() == item.id())
        {
            Some(index) => self.items[index] = item,
            None => self.items.push(item),
        }
    }

    fn remove(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item.id() != id);
        self.items.len() != before
    }

    fn all(&self) -> Vec<&T> {
        self.items.iter().collect()
    }

    fn find(&self, predicate: impl Fn(&T) -> bool) -> Vec<&T> {
        self.items.iter().filter(|item| predicate(item)).collect()
    }
}

/// A clock you can move forward by hand.
///
/// Rust: `Cell` gives *interior mutability*, so `advance_days` takes `&self`. That lets the
/// service hold a shared reference to the clock while the demo still moves time forward.
#[derive(Debug)]
pub struct ManualClock {
    today: Cell<Date>,
}

impl ManualClock {
    pub fn new(start: Date) -> Self {
        Self {
            today: Cell::new(start),
        }
    }

    /// Move time forward.
    pub fn advance_days(&self, days: i64) {
        self.today.set(self.today.get().add_days(days));
    }
}

impl Clock for ManualClock {
    fn today(&self) -> Date {
        self.today.get()
    }
}

/// `$0.75`.
pub fn format_money(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

/// Composition: the service *has* repositories, a clock and a policy.
///
/// Generics (`I`, `M`, `L`) are resolved at compile time (static dispatch); the clock is a
/// `&dyn Clock` (dynamic dispatch) borrowed for lifetime `'a`, so it must outlive the service.
pub struct LibraryService<'a, I, M, L> {
    items: I,
    members: M,
    loans: L,
    clock: &'a dyn Clock,
    policy: Policy,
}

impl<'a, I, M, L> LibraryService<'a, I, M, L>
where
    I: Repository<Box<dyn LibraryItem>>,
    M: Repository<Member>,
    L: Repository<Loan>,
{
    pub fn new(items: I, members: M, loans: L, clock: &'a dyn Clock, policy: Policy) -> Self {
        Self {
            items,
            members,
            loans,
            clock,
            policy,
        }
    }

    /// Lend an item to a member if the rules allow it.
    pub fn check_out(&mut self, item_id: &str, member_id: &str) -> Result<Checkout, LibraryError> {
        let item = self
            .items
            .get(item_id)
            .ok_or_else(|| LibraryError::ItemNotFound(item_id.to_string()))?;
        let member = self
            .members
            .get(member_id)
            .ok_or_else(|| LibraryError::MemberNotFound(member_id.to_string()))?;

        if self.loans.get(item_id).is_some() {
            return Err(LibraryError::AlreadyOnLoan(item.title().to_string()));
        }
        if self.loans.find(|loan| loan.member_id == member_id).len() >= self.policy.max_loans {
            return Err(LibraryError::LimitReached {
                name: member.name.clone(),
                limit: self.policy.max_loans,
            });
        }

        // Polymorphism: `loan_days` is dispatched through the trait object's vtable.
        let due_on = self.clock.today().add_days(i64::from(item.loan_days()));
        self.loans.add(Loan {
            id: item_id.to_string(),
            member_id: member_id.to_string(),
            due_on,
        });
        Ok(Checkout {
            title: item.title().to_string(),
            member_name: member.name.clone(),
            due_on,
        })
    }

    /// Take an item back and compute any late fee.
    pub fn return_item(&mut self, item_id: &str) -> Result<Receipt, LibraryError> {
        let item = self
            .items
            .get(item_id)
            .ok_or_else(|| LibraryError::ItemNotFound(item_id.to_string()))?;
        let loan = self
            .loans
            .get(item_id)
            .ok_or_else(|| LibraryError::NotOnLoan(item.title().to_string()))?;

        let days_late =
            u32::try_from(self.clock.today().days_since(loan.due_on).max(0)).unwrap_or(u32::MAX);
        let receipt = Receipt {
            title: item.title().to_string(),
            days_late,
            fee_cents: days_late * self.policy.fee_per_day_cents,
        };
        self.loans.remove(item_id);
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests;
