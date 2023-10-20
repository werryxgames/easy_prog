use std::{rc::Rc, fs, io::Error};

use crate::{types::{SequenceNode, NodeType, Scope, CallFuncNode, Variant, Function, NativeException, VariableNode, Void}, parser::parse};

#[derive(Debug)]
pub struct RunnerError {
    pub line: u32,
    pub column: u32,
    pub description: String
}

impl RunnerError {
    pub fn new(line: u32, column: u32, description: &str) -> RunnerError {
        return RunnerError { line, column, description: description.to_string() };
    }
}

pub fn get_variable(scope: &mut Scope, node: VariableNode) -> Result<Rc<dyn Variant>, RunnerError> {
    if !scope.variables.contains_key(&node.name) {
        return Err(RunnerError::new(node.line, node.column, &format!("No variable '{}' in the current scope", node.name)));
    }

    Ok(unsafe { (*scope.variables.get(&node.name).unwrap_unchecked()).clone() })
}

pub fn execute_func(scope: &mut Scope, node: CallFuncNode) -> Result<Result<Rc<dyn Variant>, NativeException>, RunnerError> {
    if !scope.functions.contains_key(&node.name) {
        return Err(RunnerError::new(node.line, node.column, &format!("No function '{}' in the current scope", node.name)));
    }

    let func: &mut Function = unsafe { (scope.functions.get(&node.name).unwrap_unchecked() as *const Function as *mut Function).as_mut().unwrap_unchecked() };
    let mut value_args: Vec<Rc<dyn Variant>> = Vec::new();

    for arg in node.args {
        match arg.get_type() {
            NodeType::Sequence => {
                value_args.push(Rc::new(Function::new(arg.as_sequence())));
            },
            NodeType::CallFunc => {
                let result = execute_func(&mut *scope, arg.as_call_func());

                if result.is_err() {
                    return unsafe { Err(result.unwrap_err_unchecked()) };
                }

                let native_result = unsafe { result.unwrap_unchecked() };

                if native_result.is_err() {
                    return unsafe { Ok(Err(native_result.unwrap_err_unchecked())) };
                }

                value_args.push(unsafe { native_result.unwrap_unchecked() });
            },
            NodeType::ConstInt => {
                value_args.push(Rc::new(arg.as_int_const().value));
            },
            NodeType::ConstStr => {
                value_args.push(Rc::new(arg.as_str_const().value));
            },
            NodeType::Identifier => {
                let result = get_variable(&mut *scope, arg.as_variable());

                if result.is_err() {
                    return Err(unsafe { result.unwrap_err_unchecked() });
                }

                value_args.push(unsafe { result.unwrap_unchecked() });
            }
        }
    }

    if func.body.is_some() {
        execute_sequence(&mut *scope, unsafe { func.body.as_ref().unwrap_unchecked() });
        return Ok(Ok(Rc::new(Void::new())));
    }
    else if func.native.is_some() {
        let var = unsafe { func.native.unwrap_unchecked() }(node.line, node.column, scope, value_args);
        return Ok(var);
    }
    
    return Err(RunnerError::new(node.line, node.column, &format!("Function '{}' defined in current scope, but neither native nor custom", node.name)));
}

pub fn execute_sequence(scope: &mut Scope, node: &SequenceNode) -> Option<Result<RunnerError, NativeException>> {
    for child in node.body.iter() {
        if child.get_type() != NodeType::CallFunc {
            panic!();
        }

        let result = execute_func(&mut *scope, child.as_call_func());

        if result.is_err() {
            return Some(Ok(unsafe { result.unwrap_err_unchecked() }));
        }

        let result2 = unsafe { result.unwrap_unchecked() };

        if result2.is_err() {
            return Some(Err(unsafe { result2.unwrap_err_unchecked() }));
        }
    }

    None
}

pub fn execute(scope: &mut Scope, ast: &SequenceNode, path: &str) -> bool {
    let exec_result = execute_sequence(&mut *scope, ast);

    if exec_result.is_some() {
        let error = unsafe { exec_result.unwrap_unchecked() };

        if error.is_ok() {
            let error2 = unsafe { error.unwrap_unchecked() };
            println!("{}: Runtime error on line {} column {}: {}", path, error2.line, error2.column, error2.description);
            return false;
        }

        let error2 = unsafe { error.unwrap_err_unchecked() };
        println!("{}: Native function exception on line {} column {}: {}", path, error2.line, error2.column, error2.description);
        return false;
    }

    true
}

pub fn run_code_scope(code: &str, scope: &mut Scope) -> bool {
    let parse_result = parse(code);

    if parse_result.is_err() {
        let error = unsafe { parse_result.unwrap_err_unchecked() };
        println!("Code: Error on line {} column {}: {}", error.line, error.column, error.description);
        return false;
    }

    execute(scope, &unsafe { parse_result.unwrap_unchecked() }, "Code")
}

pub fn run_code(code: &str) -> bool {
    run_code_scope(code, &mut Scope::with_stdlib())
}

pub fn run_file_scope(path: &str, scope: &mut Scope) -> bool {
    let code: Result<String, Error> = fs::read_to_string(path);

    if code.is_err() {
        println!("{}: File error: {}", path, code.unwrap_err());
        return false;
    }

    let parse_result = parse(&code.unwrap());

    if parse_result.is_err() {
        let error = unsafe { parse_result.unwrap_err_unchecked() };
        println!("{}: Error on line {} column {}: {}", path, error.line, error.column, error.description);
        return false;
    }

    let ast: SequenceNode = unsafe { parse_result.unwrap_unchecked() };
    execute(scope, &ast, path)
}

pub fn run_file(path: &str) -> bool {
    let mut scope: Scope = Scope::with_stdlib();
    run_file_scope(path, &mut scope)
}

