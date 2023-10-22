#[cfg(feature = "repl")]
use easy_prog::repl::start_repl;
#[cfg(feature = "runner")]
use easy_prog::runner::run_file;
use std::env;

#[cfg(not(feature = "runner"))]
fn run_file(_i: &String) {
    println!("Feature 'runner' required to run specified file");
}

#[cfg(not(feature = "repl"))]
fn start_repl() {
    println!("Usage: easy_prog <path_to_file.ep>");
}

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

