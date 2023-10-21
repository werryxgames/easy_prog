use crate::{
    lexer::is_identifier_char,
    runner::run_line_scope,
    types::{Scope, Type},
};
use std::{
    cmp,
    io::{self, Write},
};

static VERSION: &str = "1.0.0";
static HISTORY_LIMIT: usize = 1024;
static COMPLETION_LIMIT: usize = 64;

#[derive(Clone)]
struct VarFuncCandidate {
    text: String,
    hint: String,
}

impl VarFuncCandidate {
    pub fn new(text: String) -> VarFuncCandidate {
        VarFuncCandidate {
            text: text.clone(),
            hint: "\x1b[37m # ".to_string() + &text + "\x1b[0m",
        }
    }
}

impl rustyline::completion::Candidate for VarFuncCandidate {
    fn display(&self) -> &str {
        &self.text
    }

    fn replacement(&self) -> &str {
        &self.text
    }
}

impl rustyline::hint::Hint for VarFuncCandidate {
    fn display(&self) -> &str {
        &self.hint
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.text)
    }
}

struct VarFuncHelper {
    scope: Scope,
}

impl VarFuncHelper {
    pub fn new(scope: &mut Scope) -> VarFuncHelper {
        VarFuncHelper {
            scope: (*scope).clone(),
        }
    }

    pub fn get_completion_list(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<VarFuncCandidate>), rustyline::error::ReadlineError> {
        let mut variants: Vec<VarFuncCandidate> = Vec::new();
        let line_part = line[..pos].to_string();
        let mut i = line_part.len(); // TODO: probably -1
        let mut line_part_iter = line_part.chars().rev();

        while i > 0
            && is_identifier_char(unsafe { line_part_iter.next().unwrap_unchecked() }, false)
        {
            i -= 1;
        }

        let word = &line_part[i..];

        for var in self.scope.variables.iter() {
            if var.0.starts_with(word) && var.0 != word {
                variants.push(VarFuncCandidate::new(var.0.to_string()));
            }
        }

        for func in self.scope.functions.iter() {
            if func.0.starts_with(word) && func.0 != word {
                variants.push(VarFuncCandidate::new(func.0.to_string()));
            }
        }

        Ok((i, variants))
    }
}

impl rustyline::completion::Completer for VarFuncHelper {
    type Candidate = VarFuncCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), rustyline::error::ReadlineError> {
        self.get_completion_list(line, pos, ctx)
    }
}

impl rustyline::hint::Hinter for VarFuncHelper {
    type Hint = VarFuncCandidate;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let list = unsafe { self.get_completion_list(line, pos, ctx).unwrap_unchecked() };
        let list_values = list.1;

        if list_values.is_empty() {
            return None;
        }

        if list_values.len() > 1 {
            let mut iter = list_values.iter();
            let mut common = unsafe { iter.next().unwrap_unchecked() }.text.clone();

            for value in iter {
                let mut i = 0;
                let val_text_cloned = value.text.clone();
                let mut val_text = val_text_cloned.chars();
                let common_t_cloned = common.clone();
                let mut common_t = common_t_cloned.chars();
                let min_val = cmp::min(value.text.len(), common.len());

                while i < min_val {
                    if val_text.next() != common_t.next() {
                        break;
                    }

                    i += 1;
                }

                common = common[..i].to_string();
            }

            common = common[pos - list.0..].to_string();

            if common.is_empty() {
                return None;
            }

            return Some(VarFuncCandidate::new(common));
        }

        let result = unsafe { list_values.get(0).unwrap_unchecked() };
        let string = result.text.clone();
        Some(VarFuncCandidate::new((*string)[pos - list.0..].to_string()))
    }
}

impl rustyline::highlight::Highlighter for VarFuncHelper {}
impl rustyline::validate::Validator for VarFuncHelper {}
impl rustyline::Helper for VarFuncHelper {}

pub struct ReplError {
    pub description: String,
}

impl ReplError {
    pub fn new(description: &str) -> ReplError {
        ReplError {
            description: description.to_string(),
        }
    }
}

pub fn start_repl_ex<T: Write>(scope: &mut Scope, out: &mut T) -> ReplError {
    let builder = rustyline::config::Builder::new();
    let builder2 = builder.max_history_size(HISTORY_LIMIT);

    if builder2.is_err() {
        return ReplError::new(&format!("Config error: {}", unsafe {
            builder2.unwrap_err_unchecked()
        }));
    }

    let config = unsafe { builder2.unwrap_unchecked() }
        .completion_type(rustyline::config::CompletionType::List)
        .completion_prompt_limit(COMPLETION_LIMIT)
        .auto_add_history(true)
        .tab_stop(4)
        .bell_style(rustyline::config::BellStyle::Audible)
        .color_mode(rustyline::config::ColorMode::Forced)
        .build();
    let editor_result =
        rustyline::Editor::<VarFuncHelper, rustyline::history::DefaultHistory>::with_config(config);

    if editor_result.is_err() {
        return ReplError::new(&format!("Editor error: {}", unsafe {
            editor_result.unwrap_err_unchecked()
        }));
    }

    let mut editor = unsafe { editor_result.unwrap_unchecked() };
    editor.set_helper(Some(VarFuncHelper::new(scope)));

    if writeln!(out, "Easy Prog interpreter v.{} by Werryx Games", VERSION).is_err() {
        return ReplError::new("Stdout write error");
    }

    loop {
        if let Ok(line) = editor.readline(">>> ") {
            let result = run_line_scope(&line, scope);

            if result.is_err() {
                let error = unsafe { result.unwrap_err_unchecked() };
                let _ = writeln!(
                    out,
                    "Error on line {} column {}: {}",
                    error.line, error.column, error.description
                );
                continue;
            }

            let result2 = unsafe { result.unwrap_unchecked() };

            if result2.is_err() {
                let error = unsafe { result2.unwrap_err_unchecked() };
                let _ = writeln!(
                    out,
                    "Error on line {} column {}: {}",
                    error.line, error.column, error.description
                );
                continue;
            }

            let final_result = unsafe { result2.unwrap_unchecked() };

            match final_result.get_type() {
                Type::Void => {}
                Type::Int => {
                    let _ = writeln!(out, "{}", final_result.as_int().number);
                }
                Type::Str => {
                    let _ = writeln!(out, "\"{}\"", final_result.as_str().text);
                }
                Type::Func => {
                    let func = final_result.as_func();

                    if func.native.is_some() {
                        let _ = writeln!(out, "<NativeFunction({:#X})>", unsafe {
                            func.native.unwrap_unchecked()
                        }
                            as usize);
                    } else {
                        let _ = writeln!(out, "<Function>");
                    }
                }
                Type::Custom => {
                    let custom = final_result.as_custom();
                    let _ = writeln!(out, "<Custom id={}>", custom.get_id());
                }
            };

            editor.set_helper(Some(VarFuncHelper::new(scope)));
            let _ = writeln!(out);
            let _ = out.flush();
        }
    }
}

pub fn start_repl_scope(scope: &mut Scope) -> ReplError {
    start_repl_ex(scope, &mut io::stdout())
}

pub fn start_repl() -> ReplError {
    start_repl_scope(&mut Scope::with_stdlib())
}
