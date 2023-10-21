use easy_prog::parser::parse;

#[test]
fn test_hello_world() {
    let node = parse("print(\"Hello, World!\")").unwrap();
    assert_eq!(node.body.len(), 1);
    assert_eq!(node.body[0].as_call_func().name, "print");
    assert_eq!(node.body[0].as_call_func().args.len(), 1);
    assert_eq!(
        node.body[0].as_call_func().args[0]
            .as_str_const()
            .value
            .text,
        "Hello, World!"
    );
}
