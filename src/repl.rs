#[cfg(feature = "repl-rustyline")]
use crate::lexer::{get_checked_tokens, is_identifier_char};
#[cfg(feature = "repl-rustyline")]
use crate::types::TokenType;
#[cfg(feature = "repl-rustyline")]
use std::cmp;

use crate::{
    runner::run_line_scope,
    types::{Scope, Str, Type},
};
use std::{
    io::{self, BufRead, Stdin, Write},
    rc::Rc,
};

static VERSION: &str = "1.0.0";
static DEFAULT_PROMPT: &str = ">>> ";
#[cfg(feature = "repl-rustyline")]
static HISTORY_LIMIT: usize = 1024;
#[cfg(feature = "repl-rustyline")]
static COMPLETION_LIMIT: usize = 64;
#[cfg(feature = "repl-rustyline")]
static EP_VARIABLE_COLOR: &str = "\x1b[37m";
#[cfg(feature = "repl-rustyline")]
static VARIABLE_COLOR: &str = "\x1b[36m";
#[cfg(feature = "repl-rustyline")]
static NATIVE_FN_COLOR: &str = "\x1b[32m";
#[cfg(feature = "repl-rustyline")]
static CUSTOM_FN_COLOR: &str = "\x1b[35m";
#[cfg(feature = "repl-rustyline")]
static DEFAULT_COLOR: &str = "\x1b[0m";

#[cfg(feature = "repl-rustyline")]
#[derive(Clone)]
struct VarFuncCandidate {
    text: String,
    hint: String,
}

#[cfg(feature = "repl-rustyline")]
impl VarFuncCandidate {
    pub fn new(text: String) -> VarFuncCandidate {
        VarFuncCandidate {
            text: text.clone(),
            hint: " # ".to_string() + &text,
        }
    }
}

#[cfg(feature = "repl-rustyline")]
impl rustyline::completion::Candidate for VarFuncCandidate {
    fn display(&self) -> &str {
        &self.text
    }

    fn replacement(&self) -> &str {
        &self.text
    }
}

#[cfg(feature = "repl-rustyline")]
impl rustyline::hint::Hint for VarFuncCandidate {
    fn display(&self) -> &str {
        &self.hint
    }

    fn completion(&self) -> Option<&str> {
        None
    }
}

#[cfg(feature = "repl-rustyline")]
struct VarFuncHelper {
    scope: Scope,
}

#[cfg(feature = "repl-rustyline")]
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

#[cfg(feature = "repl-rustyline")]
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

#[cfg(feature = "repl-rustyline")]
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

#[cfg(feature = "repl-rustyline")]
impl rustyline::highlight::Highlighter for VarFuncHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Owned("\x1b[37m".to_string() + hint + "\x1b[0m")
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        for variable in self.scope.variables.iter() {
            if variable.0 == candidate {
                if candidate.starts_with("__") {
                    return std::borrow::Cow::Owned(
                        EP_VARIABLE_COLOR.to_string() + candidate + DEFAULT_COLOR,
                    );
                }

                return std::borrow::Cow::Owned(
                    VARIABLE_COLOR.to_string() + candidate + DEFAULT_COLOR,
                );
            }
        }

        for func in self.scope.functions.iter() {
            if func.0 == candidate {
                if func.1.native.is_some() {
                    return std::borrow::Cow::Owned(
                        NATIVE_FN_COLOR.to_string() + candidate + DEFAULT_COLOR,
                    );
                }

                return std::borrow::Cow::Owned(
                    CUSTOM_FN_COLOR.to_string() + candidate + DEFAULT_COLOR,
                );
            }
        }

        std::borrow::Cow::Borrowed(candidate)
    }
}

#[cfg(feature = "repl-rustyline")]
impl rustyline::validate::Validator for VarFuncHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext<'_>,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        //return Ok(rustyline::validate::ValidationResult::Valid(Some(format!("\"{}\"", ctx.input()))));
        let tokens_result = get_checked_tokens(ctx.input());

        if tokens_result.is_err() {
            // let error = unsafe { tokens_result.unwrap_err_unchecked() };
            // Let user see error from lexer, not from rustyline
            return Ok(rustyline::validate::ValidationResult::Valid(None));
        }

        // TODO: Move bracket checker out from lexer or make lexer function, that check brackets
        // sequence, but don't check its count. Also should check that opened >= closed.

        let mut opened_brackets = 0;

        for token in unsafe { tokens_result.unwrap_unchecked() } {
            if token.token_type == TokenType::Lparen || token.token_type == TokenType::Lbrace {
                opened_brackets += 1;
            } else if token.token_type == TokenType::Rparen || token.token_type == TokenType::Rbrace
            {
                opened_brackets -= 1;
            }
        }

        if opened_brackets != 0 {
            return Ok(rustyline::validate::ValidationResult::Incomplete);
        }

        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

#[cfg(feature = "repl-rustyline")]
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

#[cfg(feature = "repl-rustyline")]
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
        .build();
    let editor_result =
        rustyline::Editor::<VarFuncHelper, rustyline::history::DefaultHistory>::with_config(config);

    if editor_result.is_err() {
        return ReplError::new(&format!("Editor error: {}", unsafe {
            editor_result.unwrap_err_unchecked()
        }));
    }

    let mut editor = unsafe { editor_result.unwrap_unchecked() };
    scope
        .variables
        .insert("__prompt".to_string(), Rc::new(Str::new(DEFAULT_PROMPT)));
    editor.set_helper(Some(VarFuncHelper::new(scope)));

    if writeln!(out, "Easy Prog interpreter v.{} by Werryx Games", VERSION).is_err() {
        return ReplError::new("Stdout write error");
    }

    loop {
        let prompt_result = scope.variables.get("__prompt");
        let prompt: String;

        if prompt_result.is_none() {
            prompt = DEFAULT_PROMPT.to_string();
        } else {
            let prompt_var = unsafe { prompt_result.unwrap_unchecked() };

            if prompt_var.get_type() != Type::Str {
                prompt = DEFAULT_PROMPT.to_string();
            } else {
                prompt = prompt_var.as_str().text;
            }
        }

        if let Ok(line) = editor.readline(&prompt) {
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
                    let _ = write!(out, "{}", final_result.as_int().number);
                }
                Type::Str => {
                    let _ = write!(out, "\"{}\"", final_result.as_str().text);
                }
                Type::Func => {
                    let func = final_result.as_func();

                    if func.native.is_some() {
                        let _ = write!(out, "<NativeFunction({:#X})>", unsafe {
                            func.native.unwrap_unchecked()
                        }
                            as usize);
                    } else {
                        let _ = write!(out, "<Function>");
                    }
                }
                Type::Custom => {
                    let custom = final_result.as_custom();
                    let _ = write!(out, "<Custom id={}>", custom.get_id());
                }
            };

            editor.set_helper(Some(VarFuncHelper::new(scope)));
            let _ = writeln!(out);
            let _ = out.flush();
        }
    }
}

#[cfg(feature = "repl-rustyline")]
pub fn start_repl_scope(scope: &mut Scope) -> ReplError {
    start_repl_ex(scope, &mut io::stdout())
}

pub fn start_default_repl<W: Write>(scope: &mut Scope, out: &mut W, in_: Stdin) -> ReplError {
    scope
        .variables
        .insert("__prompt".to_string(), Rc::new(Str::new(DEFAULT_PROMPT)));

    if writeln!(out, "Easy Prog interpreter v.{} by Werryx Games", VERSION).is_err() {
        return ReplError::new("Stdout write error");
    }

    loop {
        let prompt_result = scope.variables.get("__prompt");
        let prompt: String;

        if prompt_result.is_none() {
            prompt = DEFAULT_PROMPT.to_string();
        } else {
            let prompt_var = unsafe { prompt_result.unwrap_unchecked() };

            if prompt_var.get_type() != Type::Str {
                prompt = DEFAULT_PROMPT.to_string();
            } else {
                prompt = prompt_var.as_str().text;
            }
        }

        let _ = write!(out, "{}", prompt);
        let _ = out.flush();
        let mut string: String = String::new();

        if let Ok(_result) = in_.lock().read_line(&mut string) {
            let result = run_line_scope(&string, scope);

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
                    let _ = write!(out, "{}", final_result.as_int().number);
                }
                Type::Str => {
                    let _ = write!(out, "\"{}\"", final_result.as_str().text);
                }
                Type::Func => {
                    let func = final_result.as_func();

                    if func.native.is_some() {
                        let _ = write!(out, "<NativeFunction({:#X})>", unsafe {
                            func.native.unwrap_unchecked()
                        }
                            as usize);
                    } else {
                        let _ = write!(out, "<Function>");
                    }
                }
                Type::Custom => {
                    let custom = final_result.as_custom();
                    let _ = write!(out, "<Custom id={}>", custom.get_id());
                }
            };

            let _ = writeln!(out);
            let _ = out.flush();
        }
    }
}

#[cfg(feature = "repl-rustyline")]
pub fn start_repl() -> ReplError {
    start_repl_scope(&mut Scope::with_stdlib())
}

#[cfg(not(feature = "repl-rustyline"))]
pub fn start_repl() -> ReplError {
    start_default_repl(&mut Scope::with_stdlib(), &mut io::stdout(), io::stdin())
}
