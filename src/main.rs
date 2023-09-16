use std::{fs, env};

use easy_prog_i2::parser::parse;
use easy_prog_i2::types::Scope;

fn main() {
    let path: &str = &env::args().collect::<Vec<String>>()[1];
    let code = fs::read_to_string(path).expect("File not found");
    let mut scope = Scope::with_stdlib();
    let parse_result = parse(&mut scope, &code);

    if parse_result.is_err() {
        let error = parse_result.unwrap_err();
        println!("{}: Error on line {} column {}: {}", path, error.line, error.column, error.description);
    }
}
