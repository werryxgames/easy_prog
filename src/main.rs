use easy_prog::{repl::start_repl, runner::run_file};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        for i in args[1..].iter() {
            run_file(i);
        }
    } else {
        start_repl();
    }
}
