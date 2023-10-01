use std::{io::{self, Read}, fs};
use std::rc::Rc;

use crate::types::{Type, Scope, Function, Int, Str, NativeException, Custom, Variant, Void};

macro_rules! native_function {
    ($name: ident, $args: ident, $body: block) => {
        pub fn $name($args: Vec<Rc<dyn Variant>>) -> Result<Rc<dyn Variant>, NativeException> { $body }
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

native_function!(print, args, {
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



native_function!(printerr, args, {
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

native_function!(input, args, {
    if args.len() != 0 {
        return Err(NativeException { text: format!("This function takes 0 arguments, {} given", args.len()) });
    }

    let buffer: &mut String = &mut String::new();

    if io::stdin().read_line(buffer).is_ok() {
        return Ok(Rc::new(Str::new(buffer)));
    }

    Err(NativeException { text: "I/O error".to_string() })
});

native_function!(fopen, args, {
    if args.len() != 2 {
        return Err(NativeException { text: format!("This function takes 2 arguments, {} given", args.len()) });
    }

    if args[0].get_type() != Type::Str {
        return Err(NativeException { text: "First argument of this function should be `Str(path)`".to_string() })
    }

    if args[1].get_type() != Type::Str {
        return Err(NativeException { text: "Second argument of this function should be `Str(mode)`".to_string() });
    }

    let path = args[0].as_str().text;
    let file;

    match args[1].as_str().text.as_str() {
        "w" => file = fs::File::create(path),
        "r" => file = fs::File::open(path),
        _ => { return Err(NativeException { text: "Unexpected mode. Possible values are `r` and `w`".to_string() }); }
    };

    if file.is_err() {
        return Err(NativeException { text: "I/O error".to_string() });
    }

    Ok(Rc::new(Custom::new(0, &mut file.unwrap() as *mut fs::File as *mut () )))
});

native_function!(fread, args, {
    if args.len() != 1 {
        return Err(NativeException { text: format!("This function takes 1 argument, {} given", args.len()) });
    }

    if args[0].get_type() != Type::Custom {
        return Err(NativeException { text: "First argument of this function should be `File(path)`".to_string() });
    }

    let mut buffer: String = "".to_string();
    let custom: Custom = args[0].as_custom();

    if custom.id != 0 {
        return Err(NativeException { text: "First argument should be `File(path)`".to_string() });
    }

    let file_result = ep_unpack!(custom.ptr, fs::File, try_clone).unwrap().read_to_string(&mut buffer);

    if file_result.is_err() {
        return Err(NativeException { text: "I/O error".to_string() });
    }
    
    Ok(Rc::new(Str::new(&buffer)))
});

native_function!(fwrite, args, {
    if args.len() != 2 {
        return Err(NativeException { text: format!("This function takes 2 arguments, {} given", args.len()) });
    }

    if args[0].get_type() != Type::Custom {
        return Err(NativeException { text: "First argument of this function should be `File(path)`".to_string() });
    }

    let mut buffer: String = "".to_string();
    let custom: Custom = args[0].as_custom();

    if custom.id != 0 {
        return Err(NativeException { text: "First argument should be `File(path)`".to_string() });
    }

    let file_result = ep_unpack!(custom.ptr, fs::File, try_clone).unwrap().read_to_string(&mut buffer);

    if file_result.is_err() {
        return Err(NativeException { text: "I/O error".to_string() });
    }
    
    Ok(Rc::new(Str::new(&buffer)))
});

native_function!(parse_int, args, {
    if args.len() != 1 {
        return Err(NativeException { text: format!("This function takes 1 argument, {} given", args.len()) });
    }

    if args[0].get_type() != Type::Str {
        return Err(NativeException { text: "First argument of this function should be `Str(number)`".to_string() });
    }

    let integer = args[0].as_str().text;
    let parse_result = integer.parse::<i64>();

    if parse_result.is_err() {
        return Err(NativeException { text: "Invalid number string".to_string() });
    }

    Ok(Rc::new(Int::new(parse_result.unwrap())))
});

pub fn add_print(scope: &mut Scope) {
    let mut func = Function { native: Some(print), body: None };
    scope.functions.insert("print".to_string(), &mut func as *mut Function);
}

pub fn add_printerr(scope: &mut Scope) {
    let mut func = Function { native: Some(printerr), body: None };
    scope.functions.insert("printerr".to_string(), &mut func as *mut Function);
}

pub fn add_input(scope: &mut Scope) {
    let mut func = Function { native: Some(input), body: None };
    scope.functions.insert("input".to_string(), &mut func as *mut Function);
}

pub fn add_fopen(scope: &mut Scope) {
    let mut func = Function { native: Some(fopen), body: None };
    scope.functions.insert("fopen".to_string(), &mut func as *mut Function);
}

pub fn add_fread(scope: &mut Scope) {
    let mut func = Function { native: Some(fread), body: None };
    scope.functions.insert("fread".to_string(), &mut func as *mut Function);
}

pub fn add_fwrite(scope: &mut Scope) {
    let mut func = Function { native: Some(fwrite), body: None };
    scope.functions.insert("fwrite".to_string(), &mut func as *mut Function);
}

pub fn add_parseint(scope: &mut Scope) {
    let mut func = Function { native: Some(parse_int), body: None };
    scope.functions.insert("parse_int".to_string(), &mut func as *mut Function);
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

pub fn add_stdlib(scope: &mut Scope) {
    add_io(scope);
    add_string(scope)
}
