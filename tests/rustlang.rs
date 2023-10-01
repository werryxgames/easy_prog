use std::rc::Rc;
use easy_prog_i2::types::{AstNode, CallFuncNode, ConstStrNode, Str};

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
    let node: Rc<dyn AstNode> = Rc::new(ConstStrNode::new(Str::new("Hi")));
    let a_struct = AStruct { a: node };
    assert_eq!(a_struct.a.as_str_const().value.text, "Hi");
}

fn get_node() -> Rc<dyn AstNode> {
    let node: Rc<dyn AstNode> = Rc::new(CallFuncNode::new("print".to_string(), vec![Rc::new(
        ConstStrNode::new(Str::new("Hello, World!"))
    )]));
    node
}

#[test]
pub fn test_type_erased_ptr_return() {
    assert_eq!(get_node().as_call_func().args[0].as_str_const().value.text, "Hello, World!");
}
