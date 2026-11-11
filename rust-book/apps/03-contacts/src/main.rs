use contacts::io::{print_line, read_lines};
use contacts::logic::run_session;
use contacts::models::ContactBook;

fn main() {
    run_session(
        read_lines(),
        &mut |line| print_line(line),
        ContactBook::new(),
    );
}
