use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[cfg(feature = "stdlib")]
use crate::stdlib::add_stdlib;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Int,
    Str,
    Func,
    Custom,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NodeType {
    Sequence = 0,
    CallFunc = 1,
    ConstInt = 2,
    ConstStr = 3,
    Identifier = 4,
}

pub trait Variant {
    fn get_type(&self) -> Type;
    fn as_int(&self) -> Int;
    fn as_str(&self) -> Str;
    fn as_func(&self) -> Function;
    fn as_custom(&self) -> Rc<dyn Custom>;
    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
    fn equals(&self, other: &dyn Variant) -> bool;
}

impl Debug for dyn Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print(f)
    }
}

impl PartialEq for dyn Variant {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Int {
    pub number: i64,
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

    fn as_custom(&self) -> Rc<dyn Custom> {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.number))
    }

    fn equals(&self, other: &dyn Variant) -> bool {
        self.number == other.as_int().number
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Str {
    pub text: String,
}

impl Str {
    pub fn new(text: &str) -> Str {
        Str {
            text: text.to_string(),
        }
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
        self.to_owned()
    }

    fn as_func(&self) -> Function {
        unimplemented!()
    }

    fn as_custom(&self) -> Rc<dyn Custom> {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\"{}\"", self.text))
    }

    fn equals(&self, other: &dyn Variant) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        self.text == other.as_str().text
    }
}

pub trait Custom {
    fn get_id(&self) -> usize;
    fn custom_equals(&self, custom: &mut Rc<dyn Custom>) -> bool;

    fn repr(&self) -> Option<String> {
        None
    }

    fn get_ptr(&self) -> *mut () {
        self as *const Self as *mut ()
    }
}

impl Variant for Rc<dyn Custom> {
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

    fn as_custom(&self) -> Rc<dyn Custom> {
        self.to_owned()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let repr = self.repr();

        if repr.is_some() {
            f.write_fmt(format_args!("Custom({}, {:?})", self.get_id(), unsafe {
                repr.unwrap_unchecked()
            }))
        } else {
            f.write_fmt(format_args!("Custom({})", self.get_id()))
        }
    }

    fn equals(&self, other: &dyn Variant) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        let other_var = &mut other.as_custom();

        self.get_id() == other_var.get_id() && self.custom_equals(other_var)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Void {}

impl Void {
    pub fn new() -> Void {
        Void {}
    }
}

impl Default for Void {
    fn default() -> Self {
        Self::new()
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

    fn as_custom(&self) -> Rc<dyn Custom> {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Void")
    }

    fn equals(&self, other: &dyn Variant) -> bool {
        self.get_type() == other.get_type()
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
    fn equals(&self, other: &dyn AstNode) -> bool;
}

impl Debug for dyn AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print(f)
    }
}

impl PartialEq for dyn AstNode {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

type NativeFn = fn(
    line: u32,
    column: u32,
    scope: &mut Scope,
    args: Vec<Rc<dyn Variant>>,
) -> Result<Rc<dyn Variant>, NativeException>;

#[derive(Debug, Clone)]
pub struct Function {
    pub native: Option<NativeFn>,
    pub body: Option<SequenceNode>,
}

impl Function {
    pub fn new_native(func: NativeFn) -> Function {
        Function {
            native: Some(func),
            body: None,
        }
    }

    pub fn new(body: SequenceNode) -> Function {
        Function {
            native: None,
            body: Some(body),
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        if self.native != other.native {
            return false;
        }

        if self.body.is_some() != self.body.is_some() {
            return false;
        }

        if self.body.is_none() {
            return true;
        }

        unsafe { self.body.as_ref().unwrap_unchecked() }.body
            == unsafe { other.body.as_ref().unwrap_unchecked() }.body
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
        self.to_owned()
    }

    fn as_custom(&self) -> Rc<dyn Custom> {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.native.is_some() {
            return f.write_fmt(format_args!("NativeFunction({:?})", unsafe {
                self.native.unwrap_unchecked()
            }
                as *mut ()));
        }

        if self.body.is_some() {
            return f.write_fmt(format_args!("Function({:?})", unsafe {
                self.body.as_ref().unwrap_unchecked()
            }));
        }

        f.write_str("NullFunction")
    }

    fn equals(&self, other: &dyn Variant) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        let other_var = other.as_func();

        if self.body.is_some() != other_var.body.is_some() {
            return false;
        }

        if self.body.is_none() {
            return self.native == other_var.native;
        }

        self.native == other_var.native
            && unsafe { self.body.as_ref().unwrap_unchecked() }.body
                == unsafe { other_var.body.as_ref().unwrap_unchecked() }.body
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SequenceNode {
    pub line: u32,
    pub column: u32,
    pub body: Vec<Rc<dyn AstNode>>,
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
    };
}

impl AstNode for SequenceNode {
    fn get_type(&self) -> NodeType {
        NodeType::Sequence
    }

    fn as_sequence(&self) -> SequenceNode {
        self.to_owned()
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

    fn equals(&self, other: &dyn AstNode) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        self.as_sequence().body == other.as_sequence().body
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CallFuncNode {
    pub line: u32,
    pub column: u32,
    pub name: String,
    pub args: Vec<Rc<dyn AstNode>>,
}

impl CallFuncNode {
    pub fn new(line: u32, column: u32, name: String, args: Vec<Rc<dyn AstNode>>) -> CallFuncNode {
        CallFuncNode {
            line,
            column,
            name,
            args,
        }
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
        self.to_owned()
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

    fn equals(&self, other: &dyn AstNode) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        if self.as_call_func().name == other.as_call_func().name {
            return false;
        }

        self.as_call_func().args == other.as_call_func().args
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConstIntNode {
    pub line: u32,
    pub column: u32,
    pub value: Int,
}

impl ConstIntNode {
    pub fn new(line: u32, column: u32, value: Int) -> ConstIntNode {
        ConstIntNode {
            line,
            column,
            value,
        }
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
        self.to_owned()
    }

    fn as_variable(&self) -> VariableNode {
        unimplemented!()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.print(f)
    }

    fn equals(&self, other: &dyn AstNode) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        self.as_int_const().value == other.as_int_const().value
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConstStrNode {
    pub line: u32,
    pub column: u32,
    pub value: Str,
}

impl ConstStrNode {
    pub fn new(line: u32, column: u32, value: Str) -> ConstStrNode {
        ConstStrNode {
            line,
            column,
            value,
        }
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
        self.to_owned()
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

    fn equals(&self, other: &dyn AstNode) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        self.as_str_const().value == other.as_str_const().value
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VariableNode {
    pub line: u32,
    pub column: u32,
    pub name: String,
}

impl VariableNode {
    pub fn new(line: u32, column: u32, name: &str) -> VariableNode {
        VariableNode {
            line,
            column,
            name: name.to_string(),
        }
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
        self.to_owned()
    }

    fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }

    fn equals(&self, other: &dyn AstNode) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        self.as_variable().name == other.as_variable().name
    }
}

#[derive(Debug, Clone)]
pub struct NativeException {
    pub line: u32,
    pub column: u32,
    pub description: String,
}

impl NativeException {
    pub fn new(line: u32, column: u32, description: &str) -> NativeException {
        NativeException {
            line,
            column,
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Rc<dyn Variant>>,
    functions: HashMap<String, Function>,
    parent_scope: Option<*const Scope>,
    destructors: Vec<fn(&mut Scope)>,
}

impl Scope {
    pub fn new(
        variables: HashMap<String, Rc<dyn Variant>>,
        functions: HashMap<String, Function>,
        parent: Option<*const Scope>,
    ) -> Scope {
        Scope {
            variables,
            functions,
            parent_scope: parent,
            destructors: Vec::new(),
        }
    }

    pub fn empty() -> Scope {
        Scope::new(HashMap::new(), HashMap::new(), None)
    }

    #[cfg(feature = "stdlib")]
    pub fn with_stdlib() -> Scope {
        let mut scope = Scope::empty();
        add_stdlib(&mut scope);
        scope
    }

    #[cfg(not(feature = "stdlib"))]
    pub fn with_stdlib() -> Scope {
        Scope::empty()
    }

    pub fn from_scope(scope: &Scope) -> Scope {
        let mut new_scope = Scope::empty();
        new_scope.parent_scope = Some(scope as *const Scope);
        new_scope
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
            || (self.parent_scope.is_some()
                && unsafe { (*self.parent_scope.unwrap_unchecked()).has_variable(name) })
    }

    pub fn get_variable(&self, name: &str) -> Option<&Rc<dyn Variant>> {
        let var = self.variables.get(name);

        if var.is_some() {
            return Some(unsafe { var.unwrap_unchecked() });
        }

        if self.parent_scope.is_some() {
            return unsafe { (*self.parent_scope.unwrap_unchecked()).get_variable(name) };
        }

        None
    }

    pub fn get_variables(&self) -> HashMap<String, Rc<dyn Variant>> {
        let mut variables = self.variables.clone();

        if self.parent_scope.is_some() {
            variables.extend(unsafe { (*self.parent_scope.unwrap_unchecked()).get_variables() })
        }

        variables
    }

    pub fn set_variable(&mut self, name: &str, value: Rc<dyn Variant>) -> Option<Rc<dyn Variant>> {
        self.variables.insert(name.to_string(), value)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
            || (self.parent_scope.is_some()
                && unsafe { (*self.parent_scope.unwrap_unchecked()).has_function(name) })
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        let func = self.functions.get(name);

        if func.is_some() {
            return Some(unsafe { func.unwrap_unchecked() });
        }

        if self.parent_scope.is_some() {
            return unsafe { (*self.parent_scope.unwrap_unchecked()).get_function(name) };
        }

        None
    }

    pub fn get_functions(&self) -> HashMap<String, Function> {
        let mut functions = self.functions.clone();

        if self.parent_scope.is_some() {
            functions.extend(unsafe { (*self.parent_scope.unwrap_unchecked()).get_functions() })
        }

        functions
    }

    pub fn set_function(&mut self, name: &str, func: Function) -> Option<Function> {
        self.functions.insert(name.to_string(), func)
    }

    pub fn add_destructor(&mut self, destructor: fn(&mut Scope)) {
        self.destructors.push(destructor)
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        for destructor in self.destructors.clone() {
            destructor(self);
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Unknown,
    Identifier,
    String,
    Number,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Comma,
    _CommentUnknown,
    _CommentLine,
    _CommentBlock,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
    pub line: u32,
    pub column: u32,
}

impl Token {
    pub fn new(token_type: TokenType, line: u32, column: u32, content: &str) -> Token {
        Token {
            token_type,
            content: content.to_string(),
            line,
            column,
        }
    }
}
