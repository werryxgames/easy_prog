use std::{io::{self, Read, Write}, fs, process};
use std::rc::Rc;

use crate::{types::{Type, Scope, Function, Int, Str, NativeException, Custom, Variant, Void, SequenceNode}, runner::execute_sequence};

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

native_function!(flush_stdout, line, column, _scope, args, {
    if args.len() != 0 {
        return Err(NativeException::new(line, column, &format!("This function takes 0 arguments, {} given", args.len())));
    }

    let result = io::stdout().flush();

    if result.is_err() {
        return Err(NativeException::new(line, column, &format!("I/O error")));
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

    io::stdout().flush().unwrap();
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
        return Err(NativeException::new(line, column, "Reset custom error: Function null() is unavailable for custom types"));
    }

    Ok(Rc::new(Void::new()))
});

native_function!(if_, line, column, scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(condition)`"));
    }

    if args[1].get_type() != Type::Func {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Func(body)`"));
    }

    if args[0].as_int().number == 0 {
        return Ok(Rc::new(Void::new()));
    }

    let result = execute_sequence(scope, &args[1].as_func().body.unwrap());

    if result.is_some() {
        let result2 = unsafe { result.unwrap_unchecked() };

        if result2.is_ok() {
            let error = unsafe { result2.unwrap_unchecked() };
            return Err(NativeException::new(error.line, error.column, &error.description));
        } else {
            return unsafe { Err(result2.unwrap_err_unchecked()) };
        }
    }

    Ok(Rc::new(Void::new()))
});

native_function!(if_else, line, column, scope, args, {
    if args.len() != 3 {
        return Err(NativeException::new(line, column, &format!("This function takes 3 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(condition)`"));
    }

    if args[1].get_type() != Type::Func {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Func(if_branch)`"));
    }

    if args[2].get_type() != Type::Func {
        return Err(NativeException::new(line, column, "Third argument of this function should be `Func(else_branch)`"));
    }

    if args[0].as_int().number == 0 {
        let result = execute_sequence(scope, &args[2].as_func().body.unwrap());

        if result.is_some() {
            let result2 = unsafe { result.unwrap_unchecked() };

            if result2.is_ok() {
                let error = unsafe { result2.unwrap_unchecked() };
                return Err(NativeException::new(error.line, error.column, &error.description));
            } else {
                return unsafe { Err(result2.unwrap_err_unchecked()) };
            }
        }

        return Ok(Rc::new(Void::new()));
    }

    let result = execute_sequence(scope, &args[1].as_func().body.unwrap());

    if result.is_some() {
        let result2 = unsafe { result.unwrap_unchecked() };

        if result2.is_ok() {
            let error = unsafe { result2.unwrap_unchecked() };
            return Err(NativeException::new(error.line, error.column, &error.description));
        } else {
            return unsafe { Err(result2.unwrap_err_unchecked()) };
        }
    }

    Ok(Rc::new(Void::new()))
});

native_function!(add, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(a)`"));
    }

    if args[1].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Int(b)`"));
    }

    Ok(Rc::new(Int::new(args[0].as_int().number.wrapping_add(args[1].as_int().number))))
});

native_function!(subt, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(a)`"));
    }

    if args[1].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Int(b)`"));
    }

    Ok(Rc::new(Int::new(args[0].as_int().number.wrapping_sub(args[1].as_int().number))))
});

native_function!(mult, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(a)`"));
    }

    if args[1].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Int(b)`"));
    }

    Ok(Rc::new(Int::new(args[0].as_int().number.wrapping_mul(args[1].as_int().number))))
});

native_function!(idiv, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(a)`"));
    }

    if args[1].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Int(b)`"));
    }

    let b = args[1].as_int().number;

    if b == 0 {
        return Err(NativeException::new(line, column, "Division by zero"));
    }

    Ok(Rc::new(Int::new(args[0].as_int().number / b)))
});

native_function!(and, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(a)`"));
    }

    if args[1].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Int(b)`"));
    }

    Ok(Rc::new(Int::new((args[0].as_int().number != 0 && args[1].as_int().number != 0) as i64)))
});

native_function!(or, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    if args[0].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "First argument of this function should be `Int(a)`"));
    }

    if args[1].get_type() != Type::Int {
        return Err(NativeException::new(line, column, "Second argument of this function should be `Int(b)`"));
    }

    Ok(Rc::new(Int::new((args[0].as_int().number != 0 || args[1].as_int().number != 0) as i64)))
});

native_function!(eq, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    let a = &args[0];
    let b = &args[1];
    let a_type = a.get_type();

    if a_type != b.get_type() {
        return Ok(Rc::new(Int::new(0)));
    }

    Ok(Rc::new(Int::new((a == b) as i64)))
});

native_function!(neq, line, column, _scope, args, {
    if args.len() != 2 {
        return Err(NativeException::new(line, column, &format!("This function takes 2 arguments, {} given", args.len())));
    }

    let a = &args[0];
    let b = &args[1];
    let a_type = a.get_type();

    if a_type != b.get_type() {
        return Ok(Rc::new(Int::new(0)));
    }

    Ok(Rc::new(Int::new((a != b) as i64)))
});

native_function!(exit, line, column, _scope, args, {
    let exit_code: i32;
    let args_len = args.len();

    if args_len == 0 {
        exit_code = 0i32;
    } else if args_len == 1 {
        if args[0].get_type() != Type::Int {
            return Err(NativeException::new(line, column, &format!("First argument of this function should be `Int(code)`")));
        }

        exit_code = args[0].as_int().number as i32;
    } else {
        return Err(NativeException::new(line, column, &format!("This function takes at most 1 argument, {} given", args.len())));
    }

    process::exit(exit_code)
});

native_function!(inspect_scope, line, column, scope, args, {
    if args.len() != 0 {
        return Err(NativeException::new(line, column, &format!("This function takes 0 arguments, {} given", args.len())));
    }

    println!("Begin of inspection");

    for variable in scope.variables.iter() {
        println!("Variable {} = {:?}", variable.0, variable.1);
    }

    for function in scope.functions.iter() {
        println!("Function {} = {:?}", function.0, function.1);
    }

    println!("End of inspection");
    Ok(Rc::new(Void::new()))
});

pub fn add_print(scope: &mut Scope) {
    let func = Function::new_native(print);
    scope.functions.insert("print".to_string(), func);
}

pub fn add_flush_stdout(scope: &mut Scope) {
    let func = Function::new_native(print);
    scope.functions.insert("flush_stdout".to_string(), func);
}

pub fn add_printerr(scope: &mut Scope) {
    let func = Function::new_native(printerr);
    scope.functions.insert("printerr".to_string(), func);
}

pub fn add_input(scope: &mut Scope) {
    let func = Function::new_native(input);
    scope.functions.insert("input".to_string(), func);
}

pub fn add_fopen(scope: &mut Scope) {
    let func = Function::new_native(fopen);
    scope.functions.insert("fopen".to_string(), func);
}

pub fn add_fread(scope: &mut Scope) {
    let func = Function::new_native(fread);
    scope.functions.insert("fread".to_string(), func);
}

pub fn add_fwrite(scope: &mut Scope) {
    let func = Function::new_native(fwrite);
    scope.functions.insert("fwrite".to_string(), func);
}

pub fn add_parse_int(scope: &mut Scope) {
    let func = Function::new_native(parse_int);
    scope.functions.insert("parse_int".to_string(), func);
}

pub fn add_lf(scope: &mut Scope) {
    let func = Function::new_native(lf);
    scope.functions.insert("lf".to_string(), func);
}

pub fn add_cr(scope: &mut Scope) {
    let func = Function::new_native(cr);
    scope.functions.insert("cr".to_string(), func);
}

pub fn add_declfunc(scope: &mut Scope) {
    let func = Function::new_native(declfunc);
    scope.functions.insert("declfunc".to_string(), func);
}

pub fn add_set(scope: &mut Scope) {
    let func = Function::new_native(set);
    scope.functions.insert("set".to_string(), func);
}

pub fn add_null(scope: &mut Scope) {
    let func = Function::new_native(null);
    scope.functions.insert("null".to_string(), func);
}

pub fn add_if(scope: &mut Scope) {
    let func = Function::new_native(if_);
    scope.functions.insert("if".to_string(), func);
}

pub fn add_if_else(scope: &mut Scope) {
    let func = Function::new_native(if_else);
    scope.functions.insert("if_else".to_string(), func);
}

pub fn add_add(scope: &mut Scope) {
    let func = Function::new_native(add);
    scope.functions.insert("add".to_string(), func);
}

pub fn add_subt(scope: &mut Scope) {
    let func = Function::new_native(subt);
    scope.functions.insert("subt".to_string(), func);
}

pub fn add_mult(scope: &mut Scope) {
    let func = Function::new_native(mult);
    scope.functions.insert("mult".to_string(), func);
}

pub fn add_idiv(scope: &mut Scope) {
    let func = Function::new_native(idiv);
    scope.functions.insert("idiv".to_string(), func);
}

pub fn add_and(scope: &mut Scope) {
    let func = Function::new_native(and);
    scope.functions.insert("and".to_string(), func);
}

pub fn add_or(scope: &mut Scope) {
    let func = Function::new_native(or);
    scope.functions.insert("or".to_string(), func);
}

pub fn add_eq(scope: &mut Scope) {
    let func = Function::new_native(eq);
    scope.functions.insert("eq".to_string(), func);
}

pub fn add_neq(scope: &mut Scope) {
    let func = Function::new_native(neq);
    scope.functions.insert("neq".to_string(), func);
}

pub fn add_exit(scope: &mut Scope) {
    let func = Function::new_native(exit);
    scope.functions.insert("exit".to_string(), func);
}

pub fn add_inspect_scope(scope: &mut Scope) {
    let func = Function::new_native(inspect_scope);
    scope.functions.insert("inspect_scope".to_string(), func);
}

pub fn add_stdio(scope: &mut Scope) {
    add_print(scope);
    add_flush_stdout(scope);
    add_printerr(scope);
    add_input(scope);
}

pub fn add_file_io(scope: &mut Scope) {
    add_fopen(scope);
    add_fread(scope);
    add_fwrite(scope);
}

pub fn add_io(scope: &mut Scope) {
    add_stdio(scope);
    add_file_io(scope);
}

pub fn add_string(scope: &mut Scope) {
    add_parse_int(scope);
    add_lf(scope);
    add_cr(scope);
}

pub fn add_vars(scope: &mut Scope) {
    scope.variables.insert("true".to_string(), Rc::new(Int::new(1)));
    scope.variables.insert("false".to_string(), Rc::new(Int::new(0)));
}

pub fn add_core(scope: &mut Scope) {
    add_declfunc(scope);
    add_set(scope);
    add_null(scope);
    add_if(scope);
    add_if_else(scope);
    add_add(scope);
    add_subt(scope);
    add_mult(scope);
    add_idiv(scope);
    add_and(scope);
    add_or(scope);
    add_eq(scope);
    add_neq(scope);
    add_exit(scope);
    add_vars(scope);
}

pub fn add_debug(scope: &mut Scope) {
    add_inspect_scope(scope);
}

pub fn add_stdlib(scope: &mut Scope) {
    add_io(scope);
    add_string(scope);
    add_core(scope);
    add_debug(scope);
}
