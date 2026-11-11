import {
  describeError,
  formatDate,
  formatMoney,
  InMemoryRepository,
  LibraryService,
  ManualClock,
} from "./logic.ts";
import { Book, Dvd, type LibraryItem, type Loan, type Member } from "./models.ts";

/** Scripted scenario with a fake clock so the output is identical on every run and in every port. */
export function runDemo(write: (text: string) => void): void {
  const items = new InMemoryRepository<LibraryItem>();
  items.add(new Book("b1", "Dune", "Frank Herbert"));
  items.add(new Dvd("d1", "Alien", 117));
  items.add(new Book("b2", "Emma", "Jane Austen"));

  const members = new InMemoryRepository<Member>();
  members.add({ id: "m1", name: "Ada" });
  members.add({ id: "m2", name: "Grace" });

  const clock = new ManualClock(new Date("2024-01-01T00:00:00Z"));
  const library = new LibraryService(items, members, new InMemoryRepository<Loan>(), clock);

  const checkOut = (itemId: string, memberId: string): void => {
    const result = library.checkOut(itemId, memberId);
    write(
      result.ok
        ? `Checked out '${result.value.item.title}' to ${result.value.member.name}, due ${formatDate(result.value.loan.dueOn)}`
        : `error: ${describeError(result.error)}`,
    );
  };

  const giveBack = (itemId: string): void => {
    const result = library.returnItem(itemId);
    if (!result.ok) {
      write(`error: ${describeError(result.error)}`);
      return;
    }
    const { title, daysLate, feeCents } = result.value;
    write(
      daysLate === 0
        ? `Returned '${title}' on time`
        : `Returned '${title}' ${daysLate} day${daysLate === 1 ? "" : "s"} late, fee ${formatMoney(feeCents)}`,
    );
  };

  checkOut("b1", "m1");
  checkOut("d1", "m1");
  checkOut("b2", "m1");
  checkOut("b1", "m2");
  clock.advanceDays(7);
  giveBack("d1");
  clock.advanceDays(10);
  giveBack("b1");
  giveBack("b1");
}
