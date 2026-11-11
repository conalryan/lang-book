use temperature_logger::io::{print_line, read_lines};
use temperature_logger::logic::report;

fn main() {
    for line in report(read_lines()) {
        print_line(&line);
    }
}
