use easy_prog::types::{Function, Scope};

#[test]
fn test_scope_from_scope() {
    let mut scope = Scope::with_stdlib();
    assert!(!scope.has_function(",test_func"));
    assert!(!scope.has_function(",test_func2"));
    let mut other_scope = Scope::from_scope(&scope);
    assert!(!other_scope.has_function(",test_func"));
    assert!(!other_scope.has_function(",test_func2"));

    scope.set_function(
        ",test_func",
        Function {
            native: None,
            body: None,
        },
    );
    assert!(scope.has_function(",test_func"));
    assert!(other_scope.has_function(",test_func"));

    other_scope.set_function(
        ",test_func2",
        Function {
            native: None,
            body: None,
        },
    );
    assert!(!scope.has_function(",test_func2"));
    assert!(other_scope.has_function(",test_func2"));
}
