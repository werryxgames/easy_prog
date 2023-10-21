use std::collections::VecDeque;
use std::rc::Rc;
use crate::lexer::{to_tokens, Token, TokenType};
use crate::types::{AstNode, CallFuncNode, ConstIntNode, ConstStrNode, Int, SequenceNode, Str, VariableNode};

#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum ParserErrorCode {
    EmptyTokenList,
    Lexer,
    UnexpectedType,
    ShortTokenList,
    InvalidValue
}

#[derive(Debug)]
pub struct ParserError {
    pub line: u32,
    pub column: u32,
    pub description: String,
    pub code: ParserErrorCode
}

impl ParserError {
    pub fn new(line: u32, column: u32, description: &str, code: ParserErrorCode) -> ParserError {
        return ParserError { line, column, description: description.to_string(), code };
    }
}

pub struct Vector<T: Clone> {
    mut_vec: VecDeque<T>
}

impl<T: Clone> Vector<T> {
    pub fn new(mut_vec: VecDeque<T>) -> Vector<T> {
        return Vector { mut_vec }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.mut_vec.pop_back()
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.mut_vec.pop_front()
    }

    pub fn len(&self) -> usize {
        self.mut_vec.len()
    }

    pub fn get(&self, index: usize) -> Option<T> {
        let position = index;
        self.mut_vec.get(position).cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.mut_vec.is_empty()
    }
}

/*
(* EBNF *)
letter_lowercase = "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
letter_uppercase = "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z"
digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
identifier_characters = "_"
character = "\x00" | "\x01" | "\x02" | ... | "\xFF"
comma = ","
lparen = "("
rparen = ")"
lbrace = "{"
rbrace = "}"
str_quote = '"'

string = str_quote { character - str_quote } str_quote
identifier = letter_lowercase | letter_uppercase | identifier_characters { letter_lowercase | letter_uppercase | identifier_characters | digit }
number = digit | { digit }
expression_list = ( expression { comma expression } )
func_call = identifier lparen expression_list ? rparen
func_body = lbrace expression_list ? rbrace
expression = string | identifier | number | func_call | func_body
program = START expression_list END
 */

pub fn parse_expression_list(tokens: &mut Vector<Token>, end_token: TokenType) -> Result<Vec<Rc<dyn AstNode>>, ParserError> {
    let mut nodes: Vec<Rc<dyn AstNode>> = Vec::new();
    let first_expr: Result<Rc<dyn AstNode>, ParserError> = parse_expression(tokens);

    if first_expr.is_err() {
        unsafe { return Err(first_expr.unwrap_err_unchecked()); }
    }

    nodes.push(unsafe { first_expr.unwrap_unchecked() });
    let mut last_comma: bool = false;

    while !tokens.is_empty() {
        if last_comma {
            last_comma = false;
            let expr: Result<Rc<dyn AstNode>, ParserError> = parse_expression(tokens);

            if expr.is_ok() {
                nodes.push(unsafe { expr.unwrap_unchecked() });
                continue
            }

            let error = unsafe { expr.unwrap_err_unchecked() };
            return Err(error);
        }

        let next_token = unsafe { tokens.get(0).unwrap_unchecked() };

        if next_token.token_type == TokenType::Comma {
            last_comma = true;
            tokens.pop_front();
        } else if next_token.token_type == end_token {
            return Ok(nodes);
        } else {
            return Err(ParserError::new(next_token.line, next_token.column, "Unexpected token type, expected comma (',')", ParserErrorCode::UnexpectedType));
        }
    }

    Ok(nodes)
}

pub fn parse_func_call(tokens: &mut Vector<Token>) -> Result<CallFuncNode, ParserError> {
    if tokens.len() == 0 {
        return Err(ParserError::new(0, 0, "Expected function call, but found end of file", ParserErrorCode::EmptyTokenList));
    }

    let mut token: Token = unsafe { tokens.get(0).unwrap_unchecked() };
    let first_token: &Token = &token.clone();

    if token.token_type != TokenType::Identifier {
        return Err(ParserError::new(token.line, token.column, "Unexpected token type, expected identifier", ParserErrorCode::UnexpectedType));
    }

    if tokens.len() == 1 {
        return Err(ParserError::new(0, 0, "Expected left parentheses ('('), but found end of file", ParserErrorCode::ShortTokenList));
    }

    token = unsafe { tokens.get(1).unwrap_unchecked() };

    if token.token_type != TokenType::Lparen {
        return Err(ParserError::new(token.line, token.column, "Unexpected token type, expected left parentheses ('(')", ParserErrorCode::UnexpectedType));
    }

    if tokens.len() == 2 {
        return Err(ParserError::new(0, 0, "Expected right parentheses (')'), but found end of file", ParserErrorCode::ShortTokenList));
    }

    let func_name = unsafe { tokens.pop_front().unwrap_unchecked() }.content;
    tokens.pop_front();
    token = unsafe { tokens.get(0).unwrap_unchecked() };

    if token.token_type == TokenType::Rparen {
        tokens.pop_front();
        return Ok(CallFuncNode::new(first_token.line, first_token.column, func_name, Vec::new()));
    }

    let nodes_result: Result<Vec<Rc<dyn AstNode>>, ParserError> = parse_expression_list(tokens, TokenType::Rparen);

    if nodes_result.is_err() {
        return Err(unsafe { nodes_result.unwrap_err_unchecked() })
    }

    let nodes: Vec<Rc<dyn AstNode>> = unsafe { nodes_result.unwrap_unchecked() };

    if tokens.is_empty() {
        return Err(ParserError::new(token.line, token.column, "Expected right parentheses (')'), but found end of file", ParserErrorCode::ShortTokenList));
    }

    let last_token = unsafe { tokens.pop_front().unwrap_unchecked() };

    if last_token.token_type == TokenType::Rparen {
        return Ok(CallFuncNode::new(first_token.line, first_token.column, func_name, nodes));
    }

    Err(ParserError::new(last_token.line, last_token.column, "Unexpected token type, expected right parentheses (')')", ParserErrorCode::ShortTokenList))
}

pub fn parse_func_body(tokens: &mut Vector<Token>) -> Result<SequenceNode, ParserError> {
    if tokens.len() < 2 {
        return Err(ParserError::new(0, 0, "Expected function body, but found end of file", ParserErrorCode::EmptyTokenList));
    }

    let token: &Token = &unsafe { tokens.get(0).unwrap_unchecked() };
    let first_token: &Token = token;

    if token.token_type != TokenType::Lbrace {
        return Err(ParserError::new(token.line, token.column, "Unexpected token type, expected left brace ('{')", ParserErrorCode::UnexpectedType));
    }

    tokens.pop_front();
    let result = parse_expression_list(tokens, TokenType::Rbrace);

    if result.is_err() {
        return Err(unsafe { result.unwrap_err_unchecked() });
    }

    if tokens.is_empty() {
        return Err(ParserError::new(0, 0, "Expected right brace ('}'), but found end of file", ParserErrorCode::ShortTokenList));
    }

    let last_token: Token = unsafe { tokens.pop_front().unwrap_unchecked() };

    if last_token.token_type != TokenType::Rbrace {
        return Err(ParserError::new(token.line, token.column, "Unexpected token type, expected right brace ('}')", ParserErrorCode::UnexpectedType));
    }

    Ok(SequenceNode::new(first_token.line, first_token.column, unsafe { result.unwrap_unchecked() }))
}

pub fn parse_expression(tokens: &mut Vector<Token>) -> Result<Rc<dyn AstNode>, ParserError> {
    if tokens.len() == 0 {
        return Err(ParserError::new(0, 0, "Expected expression, but found end of file", ParserErrorCode::EmptyTokenList));
    }

    let token: &Token = &unsafe { tokens.get(0).unwrap_unchecked() };

    if token.token_type == TokenType::String {
        tokens.pop_front();
        let ast_box: Rc<dyn AstNode> = Rc::new(ConstStrNode::new(token.line, token.column, Str::new(&token.content)));
        return Ok(ast_box);
    }

    if token.token_type == TokenType::Number {
        tokens.pop_front();
        let result = token.content.parse();

        if result.is_err() {
            return Err(ParserError::new(token.line, token.column, "Integer value overflowed", ParserErrorCode::InvalidValue));
        }

        let ast_box: Rc<dyn AstNode> = Rc::new(ConstIntNode::new(token.line, token.column, Int::new(unsafe { result.unwrap_unchecked() })));
        return Ok(ast_box);
    }

    if token.token_type == TokenType::Identifier {
        if tokens.len() >= 2 {
            if unsafe { tokens.get(1).unwrap_unchecked() }.token_type == TokenType::Lparen {
                let result: Result<CallFuncNode, ParserError> = parse_func_call(tokens);

                if result.is_ok() {
                    let call_func_node = unsafe { result.unwrap_unchecked() };
                    let ast_box: Rc<dyn AstNode> = Rc::new(call_func_node);
                    return Ok(ast_box)
                }

                return Err(unsafe { result.unwrap_err_unchecked() });
            }
        }

        tokens.pop_front();
        let ast_box: Rc<dyn AstNode> = Rc::new(VariableNode::new(token.line, token.column, &token.content));
        return Ok(ast_box);
    }

    if token.token_type == TokenType::Lbrace {
        let node: Result<SequenceNode, ParserError> = parse_func_body(tokens);

        if node.is_err() {
            return Err(unsafe { node.unwrap_err_unchecked() });
        }

        let ast_box: Rc<dyn AstNode> = Rc::new(unsafe { node.unwrap_unchecked() });
        return Ok(ast_box);
    }

    return Err(ParserError::new(token.line, token.column, "Unexpected token type, expected expression", ParserErrorCode::UnexpectedType));
}

pub fn parse_program(tokens: &mut Vector<Token>) -> Result<SequenceNode, ParserError> {
    let result = parse_expression_list(tokens, TokenType::Unknown);

    if result.is_err() {
        return Err(unsafe { result.unwrap_err_unchecked() })
    }

    return Ok(SequenceNode::new(1, 1, unsafe { result.unwrap_unchecked() }));
}

pub fn parse(code: &str) -> Result<SequenceNode, ParserError> {
    let tokens_result = to_tokens(code);

    if tokens_result.is_err() {
        let error = unsafe { tokens_result.unwrap_err_unchecked() };
        return Err(ParserError { line: error.line, column: error.column, description: error.description, code: ParserErrorCode::Lexer });
    }

    let tokens: VecDeque<Token> = VecDeque::from(unsafe { tokens_result.unwrap_unchecked() });
    parse_program(&mut Vector::new(tokens))
}
