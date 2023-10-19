use std::{io::{self, Read, Write}, fs};
use std::rc::Rc;

use crate::types::{Type, Scope, Function, Int, Str, NativeException, Custom, Variant, Void, SequenceNode};

macro_rules! native_function {
    ($name: ident, $line: ident, $column: ident, $scope: ident, $args: ident, $body: block) => {
        pub fn $name($line: u32, $column: u32, $scope: &mut Scope, $args: Vec<Rc<dyn Variant>>) -> Result<Rc<dyn Variant>, NativeException> { $body }
    };
}

macro_rules! ep_unpack {
    ($ptr: expr, $as_type: ty) => {
        unsafe { *($ptr as *mut $as_type) }
    };

    ($ptr: expr, $as_type: ty, $with_func: ident) => {
        unsafe { (*($ptr as *mut $as_type)).$with_func() }
    };
}

native_function!(print, _line, _column, _scope, args, {
    for arg in args {
        match arg.get_type() {
            Type::Int => { print!("{}", arg.as_int().number); },
            Type::Str => { print!("{}", arg.as_str().text); },
            Type::Void => { print!("<null>"); },
            Type::Func => { print!("<function at address {:#}>", &arg.as_func() as *const Function as u64) },
            Type::Custom => {
                let node = arg.as_custom();
                print!("<custom type {} at address {:#}>", node.id, node.ptr as u64);
            }
        }
    }

    Ok(Rc::new(Void::new()))
});

native_function!(printerr, _line, _column, _scope, args, {
    for arg in args {
        match arg.get_type() {
            Type::Int => { eprint!("{}", arg.as_int().number); },
            Type::Str => { eprint!("{}", arg.as_str().text); },
            Type::Void => { eprint!("<null>"); },
            Type::Func => { eprint!("<function at address {:#}>", &arg.as_func() as *const Function as u64) },
            Type::Custom => {
                let node = arg.as_custom();
                eprint!("<custom type {} at address {:#}>", node.id, node.ptr as u64);
            }
        }
    }

    Ok(Rc::new(Void::new()))
});

native_function!(input, line, column, _scope, args, {
    if args.len() != 0 {
        return Err(NativeException::new(line, column, &format!("This function takes 0 arguments, {} given", args.len())));
    }

    let buffer: &mut String = &mut String::new();

    if io::stdin().read_line(buffer).is_ok() {
        if buffer.ends_with("\n") {
            return Ok(Rc::new(Str::new(&buffer[..buffer.len() - 1])));
        }

        return Ok(Rc::new(Str::new(buffer)));
    }

    Err(NativeException::new(line, column, "I/O error"))
});

native_function!(fopen, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Str {
        return Err(NativeException::new(line, column, "First argument of this function should be `Str(path)`"));
    }

    if args[1].get_type() != Type::Str {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Str(mode)`"));
    }

    let path = args[0].as_str().text;
    let file;

    match args[1].as_str().text.as_str() {
        "w" => file = fs::File::create(path),
        "r" => file = fs::File::open(path),
        _ => { return Err(NativeException::new(line, column, "Unexpected mode. Possible values are `r` and `w`")); }
    };

    if file.is_err() {
        return Err(NativeException::new(line, column, "I/O error"));
    }

    Ok(Rc::new(Custom::new(0, &mut file.unwrap() as *mut fs::File as *mut () )))
});

native_function!(fread, line, column, _scope, args, {
    if args.len() != 1 {
        return Err(NativeException::new(line, column, &format!("This function takes 1 argument, {} given", args.len())));
    }

    if args[0].get_type() != Type::Custom {
        return Err(NativeException::new(line, column, "First argument of this function should be `File(path)`"));
    }

    let mut buffer: String = "".to_string();
    let custom: Custom = args[0].as_custom();

    if custom.id != 0 {
        return Err(NativeException::new(line, column, "First argument should be `File(path)`"));
    }

    let file_result = ep_unpack!(custom.ptr, fs::File, try_clone).unwrap().read_to_string(&mut buffer);

    if file_result.is_err() {
        return Err(NativeException::new(line, column, "I/O error"));
    }
    
    Ok(Rc::new(Str::new(&buffer)))
});

native_function!(fwrite, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Custom {
        return Err(NativeException::new(line, column, "First argument of this function should be `File(path)`"));
    }

    if args[1].get_type() != Type::Str {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Str(data)`"));
    }

    let custom: Custom = args[0].as_custom();

    if custom.id != 0 {
        return Err(NativeException::new(line, column, "First argument should be `File(path)`"));
    }

    let file_result = ep_unpack!(custom.ptr, fs::File, try_clone).unwrap().write_all(args[1].as_str().text.as_bytes());

    if file_result.is_err() {
        return Err(NativeException::new(line, column, "I/O error"));
    }

    Ok(Rc::new(Void::new()))
});

native_function!(parse_int, line, column, _scope, args, {
    if args.len() != 1 {
        return Err(NativeException::new(line, column, &format!("This function takes 1 argument, {} given", args.len())));
    }

    if args[0].get_type() != Type::Str {
        return Err(NativeException::new(line, column, "First argument of this function should be `Str(number)`"));
    }

    let integer = args[0].as_str().text;
    let parse_result = integer.parse::<i64>();

    if parse_result.is_err() {
        return Err(NativeException::new(line, column, "Invalid number string"));
    }

    Ok(Rc::new(Int::new(parse_result.unwrap())))
});

native_function!(lf, line, column, _scope, args, {
    if args.len() != 0 {
        return Err(NativeException::new(line, column, &format!("This function takes 0 arguments, {} given", args.len())));
    }

    Ok(Rc::new(Str::new("\n")))
});

native_function!(cr, line, column, _scope, args, {
    if args.len() != 0 {
        return Err(NativeException::new(line, column, &format!("This function takes 0 arguments, {} given", args.len())));
    }

    Ok(Rc::new(Str::new("\r")))
});

native_function!(declfunc, line, column, scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Str {
        return Err(NativeException::new(line, column, "First argument of this function should be `Str(name)`"));
    }

    if args[1].get_type() != Type::Func {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Func(body)`"));
    }

    scope.functions.insert(args[0].as_str().text, args[1].as_func());
    Ok(Rc::new(Void::new()))
});

native_function!(set, line, column, scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Str {
        return Err(NativeException::new(line, column, "First argument of this function should be `Str(name)`"));
    }

    scope.variables.insert(args[0].as_str().text, args[1].clone());
    Ok(Rc::new(Void::new()))
});

native_function!(null, line, column, scope, args, {
    if args.len() != 1 {
        return Err(NativeException::new(line, column, &format!("This function takes 1 argument, {} given", args.len())));
    }

    if args[0].get_type() != Type::Str {
        return Err(NativeException::new(line, column, "First argument of this function should be `Str(name)`"));
    }

    let var_name = args[0].as_str().text;
    let var_result = scope.variables.get(&var_name);

    if var_result.is_none() {
        return Err(NativeException::new(line, column, &format!("Variable '{}' isn't found in the current scope", var_name)));
    }

    let var = unsafe { var_result.unwrap_unchecked() };
    let var_type = var.get_type();

    if var_type == Type::Int {
        scope.variables.insert(var_name, Rc::new(Int::new(0)));
    } else if var_type == Type::Str {
        scope.variables.insert(var_name, Rc::new(Str::new("")));
    } else if var_type == Type::Func {
        scope.variables.insert(var_name, Rc::new(Function::new(SequenceNode::new(line, column, Vec::new()))));
    } else if var_type == Type::Custom {
        return Err(NativeException::new(line, column, "Reset custom error: Function null() is inavailable for custom types"));
    }

    Ok(Rc::new(Void::new()))
});

native_function!(inspect_scope, line, column, scope, args, {
    if args.len() != 0 {
        return Err(NativeException::new(line, column, &format!("This function takes 0 arguments, {} given", args.len())));
    }

    println!("Begin of inspection");

    for variable in scope.variables.clone() {
        println!("Variable {} = {:?}", variable.0, variable.1);
    }

    for function in scope.functions.clone() {
        println!("Function {} = {:?}", function.0, function.1);
    }

    println!("End of inspection");
    Ok(Rc::new(Void::new()))
});

pub fn add_print(scope: &mut Scope) {
    let func = Function { native: Some(print), body: None };
    scope.functions.insert("print".to_string(), func);
}

pub fn add_printerr(scope: &mut Scope) {
    let func = Function { native: Some(printerr), body: None };
    scope.functions.insert("printerr".to_string(), func);
}

pub fn add_input(scope: &mut Scope) {
    let func = Function { native: Some(input), body: None };
    scope.functions.insert("input".to_string(), func);
}

pub fn add_fopen(scope: &mut Scope) {
    let func = Function { native: Some(fopen), body: None };
    scope.functions.insert("fopen".to_string(), func);
}

pub fn add_fread(scope: &mut Scope) {
    let func = Function { native: Some(fread), body: None };
    scope.functions.insert("fread".to_string(), func);
}

pub fn add_fwrite(scope: &mut Scope) {
    let func = Function { native: Some(fwrite), body: None };
    scope.functions.insert("fwrite".to_string(), func);
}

pub fn add_parseint(scope: &mut Scope) {
    let func = Function { native: Some(parse_int), body: None };
    scope.functions.insert("parse_int".to_string(), func);
}

pub fn add_declfunc(scope: &mut Scope) {
    let func = Function { native: Some(declfunc), body: None };
    scope.functions.insert("declfunc".to_string(), func);
}

pub fn add_set(scope: &mut Scope) {
    let func = Function { native: Some(set), body: None };
    scope.functions.insert("set".to_string(), func);
}

pub fn add_inspect_scope(scope: &mut Scope) {
    let func = Function { native: Some(inspect_scope), body: None };
    scope.functions.insert("inspect_scope".to_string(), func);
}

pub fn add_stdio(scope: &mut Scope) {
    add_print(scope);
    add_printerr(scope);
    add_input(scope);
}

pub fn add_fileio(scope: &mut Scope) {
    add_fopen(scope);
    add_fread(scope);
    add_fwrite(scope);
}

pub fn add_io(scope: &mut Scope) {
    add_stdio(scope);
    add_fileio(scope);
}

pub fn add_string(scope: &mut Scope) {
    add_parseint(scope);
}

pub fn add_core(scope: &mut Scope) {
    add_declfunc(scope);
    add_set(scope);
}

pub fn add_debug(scope: &mut Scope) {
    add_inspect_scope(scope);
}

pub fn add_vars(scope: &mut Scope) {
    scope.variables.insert("true".to_string(), Rc::new(Int::new(1)));
    scope.variables.insert("false".to_string(), Rc::new(Int::new(0)));
}

pub fn add_stdlib(scope: &mut Scope) {
    add_io(scope);
    add_string(scope);
    add_core(scope);
    add_debug(scope);
    add_vars(scope);
}
