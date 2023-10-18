use std::rc::Rc;

use crate::types::{SequenceNode, NodeType, Scope, CallFuncNode, Variant, Function, NativeException, VariableNode, Void};

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
                let result = execute_func(scope, arg.as_call_func());

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
                let result = get_variable(scope, arg.as_variable());

                if result.is_err() {
                    return Err(unsafe { result.unwrap_err_unchecked() });
                }

                value_args.push(unsafe { result.unwrap_unchecked() });
            }
        }
    }

    if func.body.is_some() {
        execute_sequence(scope, unsafe { func.body.as_ref().unwrap_unchecked() });
        return Ok(Ok(Rc::new(Void::new())));
    }
    else if func.native.is_some() {
        return Ok(unsafe { func.native.unwrap_unchecked() }(scope, value_args));
    }
    
    return Err(RunnerError::new(node.line, node.column, &format!("Function '{}' defined in current scope, but neither native nor custom", node.name)));
}

pub fn execute_sequence(scope: &mut Scope, node: &SequenceNode) -> Option<RunnerError> {
    for child in node.body.iter() {
        if child.get_type() != NodeType::CallFunc {
            panic!();
        }

        let result = execute_func(scope, child.as_call_func());

        if result.is_err() {
            return Some(unsafe { result.unwrap_err_unchecked() });
        }
    }

    None
}

pub fn execute(scope: &mut Scope, ast: &SequenceNode) -> Option<RunnerError> {
    execute_sequence(scope, ast)
}

