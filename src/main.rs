use std::env;
use easy_prog::{runner::run_file, repl::start_repl};

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        for i in args[1..].into_iter() {
            run_file(i);
        }
    } else {
        start_repl();
    }
}
