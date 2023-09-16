use crate::{types::{Scope, SequenceNode, NodeType, Node, CallFuncNode}, lexer::{to_tokens, Token, TokenType}};
use crate::types::GetVariableNode;

#[derive(Debug)]
pub struct ParserError {
    pub line: u32,
    pub column: u32,
    pub description: String
}

pub fn parse_arguments(scope: &mut Scope, tokens: &mut Vec<Token>) -> Result<Vec<Node>, ParserError> {
    let mut token_result = peek(tokens);

    if token_result.is_none() {
        return Err(ParserError { line: 0, column: 0, description: "Expected arguments at the end of file".to_string() });
    }

    tokens.pop();
    let mut args_started = false;
    let mut nodes: Vec<Node> = Vec::new();

    while token_result.is_some() {
        let token = token_result.unwrap();

        if token.token_type == TokenType::Lparen {
            if args_started {
                return Err(ParserError { line: token.line, column: token.column, description: "Unexpected left parentheses ('(')".to_string() });
            }

            args_started = true;
        } else if token.token_type == TokenType::Rparen {
            if !args_started {
                return Err(ParserError { line: token.line, column: token.column, description: "Unexpected right parentheses (')'). May be insert left parentheses (')') before?".to_string() });
            }

            return Ok(nodes);
        } else {
            let result: Result<*mut Node, ParserError> = match token.token_type {
                TokenType::Identifier => {
                    let func_call = parse_function_call(scope, tokens);

                    if func_call.is_ok() {
                        Ok(&func_call.unwrap() as *const CallFuncNode as *mut Node)
                    } else {
                        let node = GetVariableNode { node: Node { node_type: NodeType::Identifier }, name:  };
                    }
                }
            };
            nodes.push(parse_expression())
        }

        token_result = tokens.pop();
    }

    Err(ParserError { line: 0, column: 0, description: "Expected end of argument list, got end of file".to_string() })
}

fn peek(tokens: &mut Vec<Token>) -> Option<&Token> {
    tokens.get(tokens.len() - 1)
}

pub fn parse_function_call(scope: &mut Scope, tokens: &mut Vec<Token>) -> Result<CallFuncNode, ParserError> {
    let ident_result = peek(tokens);

    if ident_result.is_none() {
        return Err(ParserError { line: 0, column: 0, description: "Expected function call at the end of file".to_string() });
    }

    let ident = ident_result.unwrap();

    if ident.token_type != TokenType::Identifier {
        return Err(ParserError { line: ident.line, column: ident.column, description: "Expected identifier".to_string() });
    }

    tokens.pop();

    let arguments_result = parse_arguments(scope, tokens);

    if arguments_result.is_err() {
        return Err(arguments_result.unwrap_err());
    }

    let arguments = arguments_result.unwrap();
    let func_call = CallFuncNode {node: Node { node_type: NodeType::CallFunc as u8 }, name: ident.content.clone(), args: arguments};
    return Ok(func_call);
}

pub fn parse_tokens(scope: &mut Scope, tokens: &mut Vec<Token>) -> Result<SequenceNode, ParserError> {
    let nodes: Vec<*mut Node> = Vec::new();

    while tokens.len() > 0 {
        let func_call_result = parse_function_call(scope, tokens);
        
        if func_call_result.is_err() {
            return Err(func_call_result.unwrap_err());
        }

        let token_result = tokens.pop();

        if token_result.is_none() {
            break;
        }

        let token = token_result.unwrap();

        if token.token_type != TokenType::Comma {
            return Err(ParserError { line: token.line, column: token.column, description: "Expected comma".to_string() });
        }
    }

    let ast = SequenceNode { node: crate::types::Node { node_type: NodeType::Sequence as u8 }, body: nodes };

    Ok(ast)
}

pub fn parse(scope: &mut Scope, code: &str) -> Result<SequenceNode, ParserError> {
    let tokens_result = to_tokens(code);

    if tokens_result.is_err() {
        let error = tokens_result.unwrap_err();
        return Err(ParserError { line: error.line, column: error.column, description: error.description });
    }

    let mut tokens = tokens_result.unwrap();
    parse_tokens(scope, &mut tokens)
}
