use std::rc::Rc;
use easy_prog::{types::{AstNode, CallFuncNode, ConstStrNode, Str, Scope, Variant, Int}, runner::execute, parser::parse};

#[test]
pub fn test_iter() {
    let string: &str = "Hello, World!";
    assert_eq!(*string.as_bytes().iter().next().unwrap() as char, 'H');
    assert_eq!(*string.as_bytes().iter().next().unwrap() as char, 'H');
    let mut iter = string.as_bytes().iter();
    assert_eq!(*iter.next().unwrap() as char, 'H');
    assert_eq!(*iter.next().unwrap() as char, 'e');
    assert_eq!(string, "Hello, World!");
}

#[test]
pub fn test_vec_iter() {
    let vec: Vec<u8> = vec![5, 3, 93, 123];
    assert_eq!(*vec.iter().next().unwrap(), 5);
    assert_eq!(*vec.iter().next().unwrap(), 5);
    let mut iter = vec.iter();
    assert_eq!(*iter.next().unwrap(), 5);
    assert_eq!(*iter.next().unwrap(), 3);
    assert_eq!(vec, vec![5, 3, 93, 123]);
}

struct AStruct {
    a: Rc<dyn AstNode>
}

#[test]
pub fn test_type_erased_ptr() {
    let node: Rc<dyn AstNode> = Rc::new(ConstStrNode::new(0, 0, Str::new("Hi")));
    let a_struct = AStruct { a: node };
    assert_eq!(a_struct.a.as_str_const().value.text, "Hi");
}

fn get_node() -> Rc<dyn AstNode> {
    let node: Rc<dyn AstNode> = Rc::new(CallFuncNode::new(0, 0, "print".to_string(), vec![Rc::new(
        ConstStrNode::new(0, 0, Str::new("Hello, World!"))
    )]));
    node
}

#[test]
pub fn test_type_erased_ptr_return() {
    assert_eq!(get_node().as_call_func().args[0].as_str_const().value.text, "Hello, World!");
}

fn set_scope_var(scope: &mut Scope, name: &str, value: Rc<dyn Variant>) {
    scope.variables.insert(name.to_string(), value);
}

#[test]
pub fn test_set_var() {
    let mut scope: Scope = Scope::with_stdlib();
    assert!(!scope.variables.contains_key(",test_var"));
    set_scope_var(&mut scope, ",test_var", Rc::new(Int::new(123)));
    assert!(scope.variables.contains_key(",test_var"));
}

#[test]
pub fn test_set_var2() {
    let mut scope: Scope = Scope::with_stdlib();
    let scope2: &mut Scope = &mut scope;
    assert!(!scope2.variables.contains_key(",test_var"));
    assert!(!scope2.variables.contains_key(",test_var2"));
    scope2.functions.get("set").unwrap().native.unwrap()(scope2, vec![Rc::new(Str::new(",test_var")), Rc::new(Int::new(123))]).unwrap();
    assert!(scope2.variables.contains_key(",test_var"));
    assert!(!scope2.variables.contains_key(",test_var2"));
}

#[test]
pub fn test_set_var3() {
    let mut scope: Scope = Scope::with_stdlib();
    assert!(!scope.variables.contains_key("___test_var"));
    assert!(!scope.variables.contains_key(",test_var2"));
    execute(&mut scope, &parse("set(\"___test_var\", parse_int(input())),print(___test_var)").unwrap());
    assert!(scope.variables.contains_key("___test_var"));
    assert!(!scope.variables.contains_key(",test_var2"));
}
