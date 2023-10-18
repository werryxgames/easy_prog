use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

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
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NodeType {
    Sequence = 0,
    CallFunc = 1,
    ConstInt = 2,
    ConstStr = 3,
    Identifier = 4
}

pub trait Variant {
    fn get_type(&self) -> Type;
    fn as_int(&self) -> Int;
    fn as_str(&self) -> Str;
    fn as_func(&self) -> Function;
    fn as_custom(&self) -> Custom;
    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
}

impl Debug for dyn Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print(f)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Int {
    pub number: i64
}

impl Int {
    pub fn new(number: i64) -> Int {
        Int { number }
    }
}

impl Variant for Int {
    fn get_type(&self) -> Type {
        Type::Int
    }

    fn as_int(&self) -> Int {
        *self
    }

    fn as_str(&self) -> Str {
        unimplemented!()
    }

    fn as_func(&self) -> Function {
        unimplemented!()
    }

    fn as_custom(&self) -> Custom {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.number))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Str {
    pub text: String
}

impl Str {
    pub fn new(text: &str) -> Str {
        Str { text: text.to_string() }
    }
}

impl Variant for Str {
    fn get_type(&self) -> Type {
        Type::Str
    }

    fn as_int(&self) -> Int {
        unimplemented!()
    }

    fn as_str(&self) -> Str {
        (*self).clone()
    }

    fn as_func(&self) -> Function {
        unimplemented!()
    }

    fn as_custom(&self) -> Custom {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\"{}\"", self.text))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Custom {
    pub id: u64,
    pub ptr: *mut ()
}

impl Custom {
    pub fn new(id: u64, ptr: *mut ()) -> Custom {
        Custom { id, ptr }
    }
}

impl Variant for Custom {
    fn get_type(&self) -> Type {
        Type::Custom
    }

    fn as_int(&self) -> Int {
        unimplemented!()
    }

    fn as_str(&self) -> Str {
        unimplemented!()
    }

    fn as_func(&self) -> Function {
        unimplemented!()
    }

    fn as_custom(&self) -> Custom {
        *self
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Custom({}, {:?})", self.id, self.ptr))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Void {}

impl Void {
    pub fn new() -> Void {
        Void {}
    }
}

impl Variant for Void {
    fn get_type(&self) -> Type {
        Type::Void
    }

    fn as_int(&self) -> Int {
        unimplemented!()
    }

    fn as_str(&self) -> Str {
        unimplemented!()
    }

    fn as_func(&self) -> Function {
        unimplemented!()
    }

    fn as_custom(&self) -> Custom {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Void")
    }
}

pub trait AstNode {
    fn get_type(&self) -> NodeType;
    fn as_sequence(&self) -> SequenceNode;
    fn as_call_func(&self) -> CallFuncNode;
    fn as_str_const(&self) -> ConstStrNode;
    fn as_int_const(&self) -> ConstIntNode;
    fn as_variable(&self) -> VariableNode;
    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
}

impl Debug for dyn AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print(f)
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub native: Option<fn(scope: &mut Scope, args: Vec<Rc<dyn Variant>>) -> Result<Rc<dyn Variant>, NativeException>>,
    pub body: Option<SequenceNode>
}

impl Function {
    pub fn new_native(func: fn(scope: &mut Scope, args: Vec<Rc<dyn Variant>>) -> Result<Rc<dyn Variant>, NativeException>) -> Function {
        Function { native: Some(func), body: None }
    }

    pub fn new(body: SequenceNode) -> Function {
        Function { native: None, body: Some(body) }
    }
}

impl Variant for Function {
    fn get_type(&self) -> Type {
        Type::Func
    }

    fn as_int(&self) -> Int {
        unimplemented!()
    }

    fn as_str(&self) -> Str {
        unimplemented!()
    }

    fn as_func(&self) -> Function {
        (*self).clone()
    }

    fn as_custom(&self) -> Custom {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.native.is_some() {
            return f.write_fmt(format_args!("NativeFunction({:?})", unsafe { self.native.unwrap_unchecked() } as *mut ()));
        }

        if self.body.is_some() {
            return f.write_fmt(format_args!("Function({:?})", unsafe { self.body.as_ref().unwrap_unchecked() }));
        }

        f.write_str("NullFunction")
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SequenceNode {
    pub line: u32,
    pub column: u32,
    pub body: Vec<Rc<dyn AstNode>>
}

impl SequenceNode {
    pub fn new(line: u32, column: u32, body: Vec<Rc<dyn AstNode>>) -> SequenceNode {
        SequenceNode { line, column, body }
    }
}

macro_rules! print_list {
    ($f: ident, $iter: ident) => {
        while $iter.len() != 0 {
            let arg = unsafe { $iter.next().unwrap_unchecked() };
            arg.print($f)?;

            if $iter.len() != 0 {
                $f.write_str(", ")?;
            }
        }
    }
}

impl AstNode for SequenceNode {
    fn get_type(&self) -> NodeType {
        NodeType::Sequence
    }

    fn as_sequence(&self) -> SequenceNode {
        (*self).clone()
    }

    fn as_call_func(&self) -> CallFuncNode {
        unimplemented!()
    }

    fn as_str_const(&self) -> ConstStrNode {
        unimplemented!()
    }

    fn as_int_const(&self) -> ConstIntNode {
        unimplemented!()
    }

    fn as_variable(&self) -> VariableNode {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Sequence(")?;
        let mut iter = self.body.iter();
        print_list!(f, iter);
        f.write_str(")")
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CallFuncNode {
    pub line: u32,
    pub column: u32,
    pub name: String,
    pub args: Vec<Rc<dyn AstNode>>
}

impl CallFuncNode {
    pub fn new(line: u32, column: u32, name: String, args: Vec<Rc<dyn AstNode>>) -> CallFuncNode {
        CallFuncNode { line, column, name, args }
    }
}

impl AstNode for CallFuncNode {
    fn get_type(&self) -> NodeType {
        NodeType::CallFunc
    }

    fn as_sequence(&self) -> SequenceNode {
        unimplemented!()
    }

    fn as_call_func(&self) -> CallFuncNode {
        (*self).clone()
    }

    fn as_str_const(&self) -> ConstStrNode {
        unimplemented!()
    }

    fn as_int_const(&self) -> ConstIntNode {
        unimplemented!()
    }

    fn as_variable(&self) -> VariableNode {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("CallFunc(\"{}\", [", self.name))?;
        let mut iter = self.args.iter();
        print_list!(f, iter);
        f.write_str("])")
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConstIntNode {
    pub line: u32,
    pub column: u32,
    pub value: Int
}

impl ConstIntNode {
    pub fn new(line: u32, column: u32, value: Int) -> ConstIntNode {
        ConstIntNode { line, column, value }
    }
}

impl AstNode for ConstIntNode {
    fn get_type(&self) -> NodeType {
        NodeType::ConstInt
    }

    fn as_sequence(&self) -> SequenceNode {
        unimplemented!()
    }

    fn as_call_func(&self) -> CallFuncNode {
        unimplemented!()
    }

    fn as_str_const(&self) -> ConstStrNode {
        unimplemented!()
    }

    fn as_int_const(&self) -> ConstIntNode {
        (*self).clone()
    }

    fn as_variable(&self) -> VariableNode {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.print(f)
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConstStrNode {
    pub line: u32,
    pub column: u32,
    pub value: Str
}

impl ConstStrNode {
    pub fn new(line: u32, column: u32, value: Str) -> ConstStrNode {
        ConstStrNode { line, column, value }
    }
}

impl AstNode for ConstStrNode {
    fn get_type(&self) -> NodeType {
        NodeType::ConstStr
    }

    fn as_sequence(&self) -> SequenceNode {
        unimplemented!()
    }

    fn as_call_func(&self) -> CallFuncNode {
        unimplemented!()
    }

    fn as_str_const(&self) -> ConstStrNode {
        (*self).clone()
    }

    fn as_int_const(&self) -> ConstIntNode {
        unimplemented!()
    }

    fn as_variable(&self) -> VariableNode {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.print(f)
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VariableNode {
    pub line: u32,
    pub column: u32,
    pub name: String
}

impl VariableNode {
    pub fn new(line: u32, column: u32, name: &str) -> VariableNode {
        VariableNode { line, column, name: name.to_string() }
    }
}

impl AstNode for VariableNode {
    fn get_type(&self) -> NodeType {
        NodeType::Identifier
    }

    fn as_sequence(&self) -> SequenceNode {
        unimplemented!()
    }

    fn as_call_func(&self) -> CallFuncNode {
        unimplemented!()
    }

    fn as_str_const(&self) -> ConstStrNode {
        unimplemented!()
    }

    fn as_int_const(&self) -> ConstIntNode {
        unimplemented!()
    }

    fn as_variable(&self) -> VariableNode {
        (*self).clone()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone)]
pub struct NativeException {
    pub text: String
}

impl NativeException {
    pub fn new(text: &str) -> NativeException {
        NativeException { text: text.to_string() }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub variables: HashMap<String, Rc<dyn Variant>>,
    pub functions: HashMap<String, Function>,
    pub parent_scope: Option<*const Scope>
}

impl Scope {
    pub fn new(variables: HashMap<String, Rc<dyn Variant>>, functions: HashMap<String, Function>, parent: Option<*const Scope>) -> Scope {
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

    pub fn get_variable(&self, name: &str) -> Option<&Rc<dyn Variant>> {
        let var = self.variables.get(name);
        
        if var.is_some() {
            return Some(var.unwrap());
        }

        if self.parent_scope.is_some() {
            return unsafe { (*self.parent_scope.unwrap()).get_variable(name) };
        }

        None
    }

    pub fn set_variable(&mut self, name: &str, value: Rc<dyn Variant>) -> Option<Rc<dyn Variant>> {
        self.variables.insert(name.to_string(), value)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name) || (self.parent_scope.is_some() && unsafe { (*self.parent_scope.unwrap()).has_function(name) })
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        let func = self.functions.get(name);
        
        if func.is_some() {
            return Some(func.unwrap());
        }

        if self.parent_scope.is_some() {
            return unsafe { (*self.parent_scope.unwrap()).get_function(name) };
        }

        None
    }

    pub fn set_function(&mut self, name: &str, func: Function) -> Option<Function> {
        self.functions.insert(name.to_string(), func)
    }
}
