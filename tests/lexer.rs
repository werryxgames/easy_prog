use easy_prog_i2::lexer::{to_tokens, Token, TokenType, LexerError};

macro_rules! test_tokens {
    ($code: expr, $tokens: expr) => {
        let tokens = to_tokens($code).unwrap();
        let result = vec_cmp(&tokens, &$tokens);

        if !result {
            println!("{:?}", tokens);
        }

        assert!(result);
    };
}

macro_rules! test_error {
    ($code: expr, $error: expr) => {
        let error = to_tokens($code).unwrap_err();
        assert_eq!(error, $error);
    };
}

fn vec_cmp<T: std::cmp::PartialEq>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool {
    let len = vec1.len();

    if len != vec2.len() {
        return false;
    }

    let mut i = 0;

    while i < len {
        if vec1[i] != vec2[i] {
            return false;
        }

        i += 1;
    }

    return true;
}

#[test]
fn test_print() {
    test_tokens!("print()", vec![
        Token::new(TokenType::Identifier, 1, 1, "print"),
        Token::new(TokenType::Lparen, 1, 6, "("),
        Token::new(TokenType::Rparen, 1, 7, ")"),
    ]);
    test_tokens!("print(123)", vec![
        Token::new(TokenType::Identifier, 1, 1, "print"),
        Token::new(TokenType::Lparen, 1, 6, "("),
        Token::new(TokenType::Number, 1, 7, "123"),
        Token::new(TokenType::Rparen, 1, 10, ")"),
    ]);
    test_tokens!("print(\"Hello, World!\")", vec![
        Token::new(TokenType::Identifier, 1, 1, "print"),
        Token::new(TokenType::Lparen, 1, 6, "("),
        Token::new(TokenType::String, 1, 7, "Hello, World!"),
        Token::new(TokenType::Rparen, 1, 22, ")"),
    ]);
    test_tokens!("print  \t\n ( 5,4,  1     ,print\t)\n", vec![
        Token::new(TokenType::Identifier, 1, 1, "print"),
        Token::new(TokenType::Lparen, 2, 2, "("),
        Token::new(TokenType::Number, 2, 4, "5"),
        Token::new(TokenType::Comma, 2, 5, ","),
        Token::new(TokenType::Number, 2, 6, "4"),
        Token::new(TokenType::Comma, 2, 7, ","),
        Token::new(TokenType::Number, 2, 10, "1"),
        Token::new(TokenType::Comma, 2, 16, ","),
        Token::new(TokenType::Identifier, 2, 17, "print"),
        Token::new(TokenType::Rparen, 2, 23, ")"),
    ]);
}

#[test]
fn test_print_error() {
    test_error!("pri nt()", LexerError { line: 1, column: 1, description: "Unexpected type after identifier".to_string() });
}

#[test]
fn test_comments() {
    // tokens = to_tokens("print() # Does nothing").unwrap();
    test_tokens!("print() # Does nothing", vec![
        Token::new(TokenType::Identifier, 1, 1, "print"),
        Token::new(TokenType::Lparen, 1, 6, "("),
        Token::new(TokenType::Rparen, 1, 7, ")"),
    ]);
    test_tokens!("## print() ## print()", vec![
        Token::new(TokenType::Identifier, 1, 15, "print"),
        Token::new(TokenType::Lparen, 1, 20, "("),
        Token::new(TokenType::Rparen, 1, 21, ")"),
    ]);
    test_tokens!("# Comment\nprint()", vec![
        Token::new(TokenType::Identifier, 2, 1, "print"),
        Token::new(TokenType::Lparen, 2, 6, "("),
        Token::new(TokenType::Rparen, 2, 7, ")"),
    ]);
    test_tokens!("## Comment\nprint() ##print()", vec![
        Token::new(TokenType::Identifier, 2, 11, "print"),
        Token::new(TokenType::Lparen, 2, 16, "("),
        Token::new(TokenType::Rparen, 2, 17, ")"),
    ]);
}
