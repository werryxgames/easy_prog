use std::{fs, env};
use std::io::Error;

use easy_prog::parser::parse;
use easy_prog::runner::execute;
use easy_prog::types::{SequenceNode, Scope};

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
    let code: Result<String, Error> = fs::read_to_string(path);

    if code.is_err() {
        println!("File error: {}", code.unwrap_err());
        return;
    }

    let parse_result = parse(&code.unwrap());
    //execute(Scope::with_stdlib(), parse_result.unwrap());

    if parse_result.is_err() {
        let error = unsafe { parse_result.unwrap_err_unchecked() };
        println!("{}: Error on line {} column {}: {}", path, error.line, error.column, error.description);
        return;
    }

    // unsafe { print_ast(parse_result.unwrap()); }
    let ast: SequenceNode = unsafe { parse_result.unwrap_unchecked() };
    let mut scope: Scope = Scope::with_stdlib();
    let exec_result = execute(&mut scope, &ast, path);
}
