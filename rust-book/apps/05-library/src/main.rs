use library::demo::run_demo;
use library::io::print_line;

fn main() {
    run_demo(&mut |line| print_line(line));
}
