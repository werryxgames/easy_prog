use crate::types::{Token, TokenType};

pub struct LexerError {
    pub line: u32,
    pub column: u32,
    pub description: String,
}

const DIGITS_FIRST: &str = "-0123456789";
const DIGITS: &str = "0123456789";
const IDENTIFIER_CHARS_FIRST: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_+-*/%$^!&~`/?:<>";
const IDENTIFIER_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_+-*/%$^!&~`/?:<>0123456789";
const WHITESPACE_CHARS: &str = " \t\n";
// const USED_CHARS: &str = "(),{}#";
// const RESERVED_CHARS: &str = "@;[]\\";

pub fn is_number_char(chr: char, is_first: bool) -> bool {
    if is_first {
        return DIGITS_FIRST.contains(chr);
    }

    DIGITS.contains(chr)
}

pub fn is_identifier_char(chr: char, is_first: bool) -> bool {
    if is_first {
        return IDENTIFIER_CHARS_FIRST.contains(chr);
    }

    IDENTIFIER_CHARS.contains(chr)
}

pub fn is_string_start(chr: char) -> bool {
    chr == '"'
}

pub fn is_string_end(chr: char) -> bool {
    chr == '"'
}

pub fn is_lparen(chr: char) -> bool {
    chr == '('
}

pub fn is_rparen(chr: char) -> bool {
    chr == ')'
}

pub fn is_lbrace(chr: char) -> bool {
    chr == '{'
}

pub fn is_rbrace(chr: char) -> bool {
    chr == '}'
}

pub fn is_comma(chr: char) -> bool {
    chr == ','
}

pub fn is_comment_start(chr: char) -> bool {
    chr == '#'
}

pub fn is_multiline_comment(chr: char, next_char: char) -> bool {
    chr == '#' && next_char != '#'
}

pub fn is_comment_end(chr: char, next_char: char, multiline: bool) -> bool {
    if multiline {
        return chr == next_char && chr == '#';
    }

    chr == '\n'
}

pub fn get_token_type(chr: char) -> TokenType {
    if is_number_char(chr, true) {
        return TokenType::Number;
    }

    if is_lparen(chr) {
        return TokenType::Lparen;
    }

    if is_rparen(chr) {
        return TokenType::Rparen;
    }

    if is_lbrace(chr) {
        return TokenType::Lbrace;
    }

    if is_rbrace(chr) {
        return TokenType::Rbrace;
    }

    if is_comma(chr) {
        return TokenType::Comma;
    }

    if is_identifier_char(chr, true) {
        return TokenType::Identifier;
    }

    TokenType::Unknown
}

pub fn get_type_start(chr: char) -> Option<TokenType> {
    if is_string_start(chr) {
        return Some(TokenType::String);
    }

    None
}

pub fn is_type_end(chr: char, token_type: &TokenType) -> bool {
    match token_type {
        TokenType::String => is_string_end(chr),
        _ => false,
    }
}

pub fn is_still_type(chr: char, token_type: &TokenType) -> bool {
    match token_type {
        TokenType::Number => is_number_char(chr, false),
        TokenType::Identifier => is_identifier_char(chr, false),
        TokenType::Lparen
        | TokenType::Rparen
        | TokenType::Lbrace
        | TokenType::Rbrace
        | TokenType::Comma => false,
        _ => true,
    }
}

pub fn is_not_continuer(token: Token) -> bool {
    token.token_type != TokenType::Lbrace
        && token.token_type != TokenType::Lparen
        && token.token_type != TokenType::Rbrace
        && token.token_type != TokenType::Rparen
        && token.token_type != TokenType::Comma
}

pub fn is_not_after_continuer(token: Token) -> bool {
    token.token_type != TokenType::Rparen
        && token.token_type != TokenType::Rbrace
        && token.token_type != TokenType::Identifier
        && token.token_type != TokenType::Number
        && token.token_type != TokenType::String
        && token.token_type != TokenType::Lbrace
}

// FIXME: Almost duplicate function
pub fn are_line_tokens_correct(tokens: &Vec<Token>) -> Result<(), LexerError> {
    if tokens.is_empty() {
        return Ok(());
    }

    if tokens[0].token_type != TokenType::Identifier {
        return Err(LexerError {
            line: 1,
            column: 1,
            description: "First element should be identifier".to_string(),
        });
    }

    let mut i = 0;
    let tokens_length = tokens.len();
    let brackets: &mut Vec<&Token> = &mut Vec::new();

    while i < tokens_length {
        let token = &tokens[i];
        i += 1;

        match token.token_type {
            TokenType::Lparen => brackets.push(token),
            TokenType::Lbrace => brackets.push(token),
            TokenType::Rparen => {
                let open_token_result = brackets.pop();

                if open_token_result.is_none() {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected closing bracket".to_string(),
                    });
                }

                let open_token = unsafe { open_token_result.unwrap_unchecked() };

                if open_token.token_type != TokenType::Lparen {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: format!(
                            "Unclosed bracket in line {} column {}",
                            open_token.line, open_token.column
                        ),
                    });
                }
            }
            TokenType::Rbrace => {
                let open_token_result = brackets.pop();

                if open_token_result.is_none() {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected closing bracket".to_string(),
                    });
                }

                let open_token = unsafe { open_token_result.unwrap_unchecked() };

                if open_token.token_type != TokenType::Lbrace {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: format!(
                            "Unclosed bracket in line {} column {}",
                            open_token.line, open_token.column
                        ),
                    });
                }
            }
            _ => continue,
        }
    }

    i = 0;

    macro_rules! check_paren {
        ($i: ident, $tokens_length: ident) => {
            if $i >= $tokens_length {
                break;
            }
        };
    }

    while i < tokens_length {
        let token = &tokens[i];
        i += 1;

        match token.token_type {
            TokenType::Identifier => {
                if i >= tokens_length {
                    break;
                }

                if is_not_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after identifier".to_string(),
                    });
                }
            }
            TokenType::Number => {
                if i >= tokens_length {
                    break;
                }

                if is_not_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after number".to_string(),
                    });
                }
            }
            TokenType::String => {
                if i >= tokens_length {
                    break;
                }

                if is_not_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after string".to_string(),
                    });
                }
            }
            TokenType::Comma => {
                check_paren!(i, tokens_length);

                if is_not_after_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after comma (',')".to_string(),
                    });
                }
            }
            TokenType::Lparen => {
                if i >= tokens_length {
                    break;
                }

                if tokens[i].token_type != TokenType::Comma
                    && is_not_after_continuer(tokens[i].clone())
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after left parentheses ('(')".to_string(),
                    });
                }
            }
            TokenType::Rparen => {
                check_paren!(i, tokens_length);

                if tokens[i].token_type != TokenType::Comma
                    && tokens[i].token_type != TokenType::Rparen
                    && tokens[i].token_type != TokenType::Rbrace
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after right parentheses (')')".to_string(),
                    });
                }
            }
            TokenType::Lbrace => {
                if i >= tokens_length {
                    break;
                }

                if tokens[i].token_type != TokenType::Comma
                    && is_not_after_continuer(tokens[i].clone())
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after left brace ('{')".to_string(),
                    });
                }
            }
            TokenType::Rbrace => {
                check_paren!(i, tokens_length);

                if tokens[i].token_type != TokenType::Comma
                    && tokens[i].token_type != TokenType::Rparen
                    && tokens[i].token_type != TokenType::Rbrace
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after right brace ('}')".to_string(),
                    });
                }
            }
            _ => {
                return Err(LexerError {
                    line: token.line,
                    column: token.column,
                    description: "Unexpected token type".to_string(),
                });
            }
        }
    }

    Ok(())
}

pub fn are_tokens_correct(tokens: &Vec<Token>) -> Result<(), LexerError> {
    // START:(Ident)
    // Ident:(Lparen...Ident=AIdent,Number=ANumber,String=AString...Rparen|Lbrace...Ident=AIdent,Number=ANumber,String=AString...Rbrace)
    // AIdent,ANumber,AString:(Rparen|Comma|Lparen|Lbrace|Rbrace)
    // Comma:(Rparen|Rbrace|AIdent|ANumber|AString|Lbrace|END)
    // Number:SHOULDN'T MET
    // Lparen,Lbrace:(AIdent|Comma|ANumber|Rparen|Rbrace|AString|Lbrace)
    // Rparen,Rbrace:(Comma|Rparen|Rbrace|END)
    // String:SHOULDN'T MET
    // Unknown:SHOULDN'T MET

    if tokens.is_empty() {
        return Ok(());
    }

    if tokens[0].token_type != TokenType::Identifier {
        return Err(LexerError {
            line: 1,
            column: 1,
            description: "First element should be identifier".to_string(),
        });
    }

    let mut i = 0;
    let tokens_length = tokens.len();
    let mut scope_level = 0;
    let brackets: &mut Vec<&Token> = &mut Vec::new();

    while i < tokens_length {
        let token = &tokens[i];
        i += 1;

        match token.token_type {
            TokenType::Lparen => brackets.push(token),
            TokenType::Lbrace => brackets.push(token),
            TokenType::Rparen => {
                let open_token_result = brackets.pop();

                if open_token_result.is_none() {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected closing bracket".to_string(),
                    });
                }

                let open_token = unsafe { open_token_result.unwrap_unchecked() };

                if open_token.token_type != TokenType::Lparen {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: format!(
                            "Unclosed bracket in line {} column {}",
                            open_token.line, open_token.column
                        ),
                    });
                }
            }
            TokenType::Rbrace => {
                let open_token_result = brackets.pop();

                if open_token_result.is_none() {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected closing bracket".to_string(),
                    });
                }

                let open_token = unsafe { open_token_result.unwrap_unchecked() };

                if open_token.token_type != TokenType::Lbrace {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: format!(
                            "Unclosed bracket in line {} column {}",
                            open_token.line, open_token.column
                        ),
                    });
                }
            }
            _ => continue,
        }
    }

    i = 0;

    macro_rules! check_paren {
        ($i: ident, $tokens_length: ident, $scope_level: ident, $token: ident) => {
            if $i >= $tokens_length {
                if $scope_level == 0 {
                    break;
                }

                return Err(LexerError {
                    line: $token.line,
                    column: $token.column,
                    description: "Unterminated left parentheses (')')".to_string(),
                });
            }
        };
    }

    while i < tokens_length {
        let token = &tokens[i];
        i += 1;

        match token.token_type {
            TokenType::Identifier => {
                if i >= tokens_length {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unterminated left parentheses (')')".to_string(),
                    });
                }

                if is_not_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after identifier".to_string(),
                    });
                }
            }
            TokenType::Number => {
                if i >= tokens_length {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unterminated left parentheses (')')".to_string(),
                    });
                }

                if is_not_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after number".to_string(),
                    });
                }
            }
            TokenType::String => {
                if i >= tokens_length {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unterminated left parentheses (')')".to_string(),
                    });
                }

                if is_not_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after string".to_string(),
                    });
                }
            }
            TokenType::Comma => {
                check_paren!(i, tokens_length, scope_level, token);

                if is_not_after_continuer(tokens[i].clone()) {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after comma (',')".to_string(),
                    });
                }
            }
            TokenType::Lparen => {
                scope_level += 1;

                if i >= tokens_length {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unterminated left parentheses (')')".to_string(),
                    });
                }

                if tokens[i].token_type != TokenType::Comma
                    && is_not_after_continuer(tokens[i].clone())
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after left parentheses ('(')".to_string(),
                    });
                }
            }
            TokenType::Rparen => {
                scope_level -= 1;

                check_paren!(i, tokens_length, scope_level, token);

                if tokens[i].token_type != TokenType::Comma
                    && tokens[i].token_type != TokenType::Rparen
                    && tokens[i].token_type != TokenType::Rbrace
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after right parentheses (')')".to_string(),
                    });
                }
            }
            TokenType::Lbrace => {
                scope_level += 1;

                if i >= tokens_length {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unterminated left parentheses (')')".to_string(),
                    });
                }

                if tokens[i].token_type != TokenType::Comma
                    && is_not_after_continuer(tokens[i].clone())
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after left brace ('{')".to_string(),
                    });
                }
            }
            TokenType::Rbrace => {
                scope_level -= 1;

                check_paren!(i, tokens_length, scope_level, token);

                if tokens[i].token_type != TokenType::Comma
                    && tokens[i].token_type != TokenType::Rparen
                    && tokens[i].token_type != TokenType::Rbrace
                {
                    return Err(LexerError {
                        line: token.line,
                        column: token.column,
                        description: "Unexpected type after right brace ('}')".to_string(),
                    });
                }
            }
            _ => {
                return Err(LexerError {
                    line: token.line,
                    column: token.column,
                    description: "Unexpected token type".to_string(),
                });
            }
        }
    }

    Ok(())
}

fn __to_tokens(code: &str) -> Result<Vec<Token>, LexerError> {
    let mut chars = code.chars();
    let mut line: u32 = 1;
    let mut column: u32 = 0;
    let mut tokens: Vec<Token> = Vec::new();
    let mut next_token: Token = Token {
        token_type: TokenType::Unknown,
        content: String::new(),
        line,
        column: column + 1,
    };
    let mut next_char: char = '\0';

    loop {
        let chr;

        if next_char == '\0' {
            let chr_result = chars.next();

            if chr_result.is_none() {
                break;
            }

            chr = unsafe { chr_result.unwrap_unchecked() };
        } else {
            chr = next_char;
            next_char = '\0';
        }

        if chr == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }

        if next_token.token_type == TokenType::_CommentUnknown {
            if next_token.content.is_empty() {
                next_token.content.push(chr);
                continue;
            }

            if is_multiline_comment(unsafe { next_token.content.pop().unwrap_unchecked() }, chr) {
                next_token.token_type = TokenType::_CommentBlock;
            } else {
                next_token.token_type = TokenType::_CommentLine;
            }

            next_token.content.clear();
            next_token.content.push(chr);
            continue;
        }

        if next_token.token_type == TokenType::_CommentLine {
            if is_comment_end(
                unsafe { next_token.content.pop().unwrap_unchecked() },
                chr,
                false,
            ) {
                next_token.token_type = TokenType::Unknown;
                next_token.content.clear();
            } else {
                next_token.content.clear();
                next_token.content.push(chr);
                continue;
            }
        }

        if next_token.token_type == TokenType::_CommentBlock {
            if is_comment_end(
                unsafe { next_token.content.pop().unwrap_unchecked() },
                chr,
                true,
            ) {
                next_token.token_type = TokenType::Unknown;
                next_token.content.clear();
            } else {
                next_token.content.clear();
                next_token.content.push(chr);
            }

            continue;
        }

        if next_token.token_type == TokenType::String && chr == '\n' {
            return Err(LexerError {
                line: next_token.line,
                column: next_token.column,
                description: "Unterminated string literal".to_string(),
            });
        }

        if next_token.token_type == TokenType::Unknown {
            if is_comment_start(chr) {
                next_token.token_type = TokenType::_CommentUnknown;
                next_token.content = String::new();
                continue;
            }

            if WHITESPACE_CHARS.contains(chr) {
                continue;
            }

            next_token.line = line;
            next_token.column = column;
            match get_type_start(chr) {
                Some(token_type) => {
                    next_token.token_type = token_type;
                    continue;
                }
                None => next_token.token_type = get_token_type(chr),
            };
        } else if is_type_end(chr, &next_token.token_type) {
            tokens.push(next_token);
            next_token = Token {
                token_type: TokenType::Unknown,
                content: String::new(),
                line: 0,
                column: 0,
            };
            continue;
        } else if !is_still_type(chr, &next_token.token_type) {
            tokens.push(next_token);
            next_token = Token {
                token_type: TokenType::Unknown,
                content: String::new(),
                line: 0,
                column: 0,
            };
            next_char = chr;

            if column == 0 && line > 0 {
                line -= 1;
            } else {
                column -= 1;
            }

            continue;
        }

        next_token.content.push(chr);
    }

    if next_token.token_type == TokenType::String {
        return Err(LexerError {
            line: next_token.line,
            column: next_token.column,
            description: "Unterminated string literal".to_string(),
        });
    }

    Ok(tokens)
}

fn _to_tokens(code: &str) -> Result<Vec<Token>, LexerError> {
    let tokens_result = __to_tokens(code);

    if tokens_result.is_err() {
        return Err(unsafe { tokens_result.unwrap_err_unchecked() });
    }

    let tokens = unsafe { tokens_result.unwrap_unchecked() };

    match are_tokens_correct(&tokens) {
        Err(error) => Err(error),
        Ok(_value) => Ok(tokens),
    }
}

pub fn check_line_error(code: &str) -> Option<LexerError> {
    let mut string = code.to_string();
    string.push('\n');
    let tokens_result = __to_tokens(&string);

    if tokens_result.is_err() {
        return Some(unsafe { tokens_result.unwrap_err_unchecked() });
    }

    let tokens = unsafe { tokens_result.unwrap_unchecked() };

    match are_line_tokens_correct(&tokens) {
        Err(error) => Some(error),
        Ok(_value) => None,
    }
}

pub fn get_checked_tokens(code: &str) -> Result<Vec<Token>, LexerError> {
    let mut string = code.to_string();
    string.push('\n');
    let tokens_result = __to_tokens(&string);

    if tokens_result.is_err() {
        return Err(unsafe { tokens_result.unwrap_err_unchecked() });
    }

    let tokens = unsafe { tokens_result.unwrap_unchecked() };

    match are_line_tokens_correct(&tokens) {
        Err(error) => Err(error),
        Ok(_value) => Ok(tokens),
    }
}

pub fn to_tokens_rev(code: &str) -> Result<Vec<Token>, LexerError> {
    let mut string = code.to_string();
    string.push('\n');
    let tokens: Result<Vec<Token>, LexerError> = _to_tokens(&string);
    tokens
}

pub fn to_tokens(code: &str) -> Result<Vec<Token>, LexerError> {
    let tokens = to_tokens_rev(code);

    if tokens.is_err() {
        return Err(unsafe { tokens.unwrap_err_unchecked() });
    }

    Ok(unsafe { tokens.unwrap_unchecked() })
}
