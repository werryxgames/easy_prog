use std::{io::{self, Write}, sync::Arc};
use crate::{runner::run_line_scope, types::{Scope, Type}};
use linefeed::{self, ReadResult, Completer, Terminal, Completion};

static VERSION: &str = "1.0.0";
static HISTORY_LIMIT: usize = 1024;

struct VarFuncCompleter {
    scope: Scope
}

unsafe impl Sync for VarFuncCompleter {}
unsafe impl Send for VarFuncCompleter {}

impl VarFuncCompleter {
    pub fn new(scope: &mut Scope) -> VarFuncCompleter {
        VarFuncCompleter { scope: (*scope).clone() }
    }
}

impl<Term: Terminal> Completer<Term> for VarFuncCompleter {
    fn complete(&self, word: &str, _prompter: &linefeed::Prompter<Term>,
                _start: usize, _end: usize) -> Option<Vec<linefeed::Completion>> {
        let mut variants: Vec<Completion> = Vec::new();

        for var in self.scope.variables.iter() {
            if var.0.starts_with(word) && var.0 != word {
                variants.push(Completion::simple(var.0.to_string()));
            }
        }

        for func in self.scope.functions.iter() {
            if func.0.starts_with(word) && func.0 != word {
                variants.push(Completion::simple(func.0.to_string()));
            }
        }
        
        if variants.is_empty() {
            return None;
        }

        Some(variants)
    }
}

pub fn start_repl_ex<T: Write>(scope: &mut Scope, out: &mut T) -> ! {
    let interface = linefeed::interface::Interface::new("Easy Prog").unwrap();
    let mut reader = interface.lock_reader();
    reader.set_history_size(HISTORY_LIMIT);
    reader.set_completion_append_character(None);
    reader.set_prompt(">>> ").unwrap();
    reader.set_completer(Arc::new(VarFuncCompleter::new(scope)));
    writeln!(out, "Easy Prog interpreter v.{} by Werryx Games", VERSION).unwrap();

    loop {
        if let ReadResult::Input(line) = reader.read_line().unwrap() {
            reader.add_history_unique(line.clone());
            let result = run_line_scope(&line, scope);

            if result.is_err() {
                let error = unsafe { result.unwrap_err_unchecked() };
                println!("Error on line {} column {}: {}", error.line, error.column, error.description);
                continue;
            }

            let result2 = unsafe { result.unwrap_unchecked() };

            if result2.is_err() {
                let error = unsafe { result2.unwrap_err_unchecked() };
                println!("Error on line {} column {}: {}", error.line, error.column, error.description);
                continue;
            }

            let final_result = unsafe { result2.unwrap_unchecked() };

            match final_result.get_type() {
                Type::Void => {},
                Type::Int => {
                    println!("{}", final_result.as_int().number)
                },
                Type::Str => {
                    println!("\"{}\"", final_result.as_str().text)
                },
                Type::Func => {
                    let func = final_result.as_func();

                    if func.native.is_some() {
                        println!("<NativeFunction({:#X})>", unsafe { func.native.unwrap_unchecked() } as usize);
                    } else {
                        println!("<Function>");
                    }
                },
                Type::Custom => {
                    let custom = final_result.as_custom();
                    println!("<Custom id={}>", custom.get_id());
                }
            };

            reader.set_completer(Arc::new(VarFuncCompleter::new(scope)));
            out.flush().unwrap();
        }
    }
}

pub fn start_repl_scope(scope: &mut Scope) {
    start_repl_ex(scope, &mut io::stdout())
}

pub fn start_repl() {
    start_repl_scope(&mut Scope::with_stdlib())
}

