use std::collections::HashMap;

use crate::stdlib::add_stdlib;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Int,
    Str,
    Func,
    Custom
}

#[repr(u8)]
#[derive(Debug)]
pub enum NodeType {
    Sequence,
    CallFunc,
    ConstInt,
    ConstStr,
    Identifier,
    Lambda
}

#[derive(Debug, Clone, Copy)]
pub struct Int {
    pub number: i64
}

#[derive(Debug, Clone)]
pub struct Str {
    pub text: String
}

#[derive(Debug, Clone, Copy)]
pub struct Custom {
    pub id: u64,
    pub ptr: *mut ()
}

#[derive(Debug)]
pub struct Value {
    pub ep_type: Type,
    pub ptr: *mut ()
}

#[derive(Debug)]
pub struct Function {
    pub native: Option<fn(args: Vec<Value>) -> Result<Value, NativeException>>,
    pub body: Option<SequenceNode>
}

pub trait NativeFunction {
    fn call(args: Vec<Value>) -> Value;
}

#[derive(Debug)]
pub struct Node {
    pub node_type: u8
}

#[derive(Debug)]
pub struct SequenceNode {
    pub node: Node,
    pub body: Vec<*mut Node>
}

#[derive(Debug)]
pub struct CallFuncNode {
    pub node: Node,
    pub name: String,
    pub args: Vec<Node>
}

#[derive(Debug)]
pub struct LambdaNode {
    pub node: Node,
    pub body: SequenceNode
}

#[derive(Debug)]
pub struct ConstIntNode {
    pub node: Node,
    pub value: Int
}

#[derive(Debug)]
pub struct ConstStrNode {
    pub node: Node,
    pub value: Str
}

#[derive(Debug)]
pub struct GetVariableNode {
    pub node: Node,
    pub name: String
}

#[derive(Debug)]
pub struct NativeException {
    pub text: String
}

#[derive(Debug)]
pub struct Scope {
    pub variables: HashMap<String, *mut Value>,
    pub functions: HashMap<String, *mut Function>,
    pub parent_scope: Option<*const Scope>
}

impl Scope {
    pub fn new(variables: HashMap<String, *mut Value>, functions: HashMap<String, *mut Function>, parent: Option<*const Scope>) -> Scope {
        Scope { variables, functions, parent_scope: parent }
    }

    pub fn empty() -> Scope {
        Scope::new(HashMap::new(), HashMap::new(), None )
    }

    pub fn with_stdlib() -> Scope {
        let mut scope = Scope::empty();
        add_stdlib(&mut scope);
        scope
    }

    pub fn from_scope(scope: &Scope) -> Scope {
        let mut new_scope = Scope::empty();
        new_scope.parent_scope = Some(scope as *const Scope);
        new_scope
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name) || (self.parent_scope.is_some() && unsafe { (*self.parent_scope.unwrap()).has_variable(name) })
    }

    pub fn get_variable(&self, name: &str) -> Option<*mut Value> {
        let var = self.variables.get(name);
        
        if var.is_some() {
            return Some(var.unwrap() as *const *mut Value as *mut Value);
        }

        if self.parent_scope.is_some() {
            return unsafe { (*self.parent_scope.unwrap()).get_variable(name) };
        }

        None
    }

    pub fn set_variable(&mut self, name: &str, value: &Value) -> Option<*mut Value> {
        self.variables.insert(name.to_string(), value as *const Value as *mut Value)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name) || (self.parent_scope.is_some() && unsafe { (*self.parent_scope.unwrap()).has_function(name) })
    }

    pub fn get_function(&self, name: &str) -> Option<*mut Function> {
        let func = self.functions.get(name);
        
        if func.is_some() {
            return Some(func.unwrap() as *const *mut Function as *mut Function);
        }

        if self.parent_scope.is_some() {
            return unsafe { (*self.parent_scope.unwrap()).get_function(name) };
        }

        None
    }

    pub fn set_function(&mut self, name: &str, func: *mut Function) -> Option<*mut Function> {
        self.functions.insert(name.to_string(), func)
    }
}
