use std::env;
use easy_prog::runner::run_file;

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>();

    if args.len() == 0 {
        println!("Usage: ./easy_prog <path to file>");
        return;
    }

    if args.len() == 1 {
        println!("Usage: {} <path to file>", args[0]);
        return;
    }

    let path: &str = &args[1];
    run_file(path);
}
