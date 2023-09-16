use crate::{types::{SequenceNode, Node}, lexer::{to_tokens, Token}};

#[derive(Debug)]
pub struct ParserError {
    pub line: u32,
    pub column: u32,
    pub description: String
}

pub fn parse_tokens(tokens: &mut Vec<Token>) -> Result<SequenceNode, ParserError> {
    /*
    # Syntax:
    START [Identifier=Id,Lparen=Lp,Rparen=Rp,Lbrace=Lb,Rbrace=Rb,Id Lp=Fc,IntConstant=Ic,StringConstant=Sc,Comma=Cm] END = SequenceNode(vec![Split(Cm, $)])
    Fc [Id=ID,Ic=IC,Sc=SC,Cm=CM,Fc..Rp=OF,Lb..Rb=FB]* Rp = CallFuncNode(Fc.Id.value, vec![Split(CM, Fc.Lp..Rp)])
        ID = GetVariableNode(ID.name)
        IC = IntConstant(IC.value)
        SC = StrConstant(SC.value)
        OF = CallFuncNode(OF.Fc.Id.name, vec![Split(CM, OF.Fc..OF.Rp)])
        FB = LambdaNode(FB.Lb..FB.Rb)
    ELSE = ParserError(...)
    # End of syntax

    # Code:
    declvar("i", "a", 5),
    declvar("i", "b", parse_int(input()))
    print(add(a, b))
    # End of code

    # Pseudo-code implementation:
    fn parse_sequence() {
        seq=vec![]
        seq.push(parse_function())
        while (expect_comma().is_some()) {
            seq.push(parse_function())
        }
        return seq;
    }
    
    fn expect_*(tokens, *=a) {
        if peek(tokens).type != a {
            return None;
        }

        return Some(tokens.pop())
    }
    
    get_identifier = expect_identifier;
    
    fn get_arguments() {
        args=vec![]
        args.push(parse_expression_or_rparen())
        while (expect_comma().is_some()) {
            args.push(parse_expression_or_rparen());
        }
        return args;
    }
    
    fn parse_expression_or_rparen(tokens) {
        let token = peek(tokens);

        if token.type == Ident&&peek_next(tokens).type==Lparen {
            return parse_function(tokens);
        } else if token.type==Ident {
            return parse_identifier(tokens);
        } else if token.type==Str {
            return parse_str(tokens);
        } else if token.type == Int {
            return parse_int(tokens);
        } else {
            return ParserError(...);
        }
    }

    fn parse_function {
        return CallFuncNode(get_identifier(), get_arguments());
    }

    parse_* = todo!();
    # End of pseudo-code implementation

    # Result
    parse_sequence(##TOPLEVEL##) {
        parse_function {
            get_identifier() = "declvar";
            get_arguments() {
                expect_lparen();
                parse_expression_or_rparen() = Str("i");
                expect_comma();
                parse_expression_or_rparen() = Str("a");
                expect_comma();
                parse_expression_or_rparen() = Int(5);
                expect_comma() = None => args = vec![Str("i"), Str("a"), Int(5)];
                expect_rparen();
                return args;
            } = vec![Str("i"), Str("a"), Int(5)];
            return all;
        } = CallFuncNode("declvar", vec![Str("i"), Str("a"), Int(5)]);
        expect_comma();
        parse_function {
            get_identifier() = "declvar";
            get_arguments() {
                expect_lparen();
                parse_expression_or_rparen() = Str("i");
                expect_comma();
                parse_expression_or_rparen() = Str("b");
                expect_comma();
                parse_expression_or_rparen() = parse_function {
                    get_identifier() = "parse_int";
                    get_arguments() {
                        expect_lparen();
                        parse_expression_or_rparen() = parse_function {
                            get_identifier() = "input";
                            get_arguments() {
                                expect_lparen();
                                parse_expression_or_rparen() = None;
                                expect_rparen();
                                return vec![];
                            } = vec![];
                        } = CallFuncNode("input", vec![]);
                    } = vec![CallFuncNode("input", vec![])];
                } = CallFuncNode("parse_int", vec![CallFuncNode("input", vec![])]);
                expect_comma() = None => args = vec![Str("i"), Str("b"), CallFuncNode("parse_int", vec![CallFuncNode("input", vec![])])];
                expect_rparen();
                return args;
            } = vec![Str("i"), Str("b"), CallFuncNode("parse_int", vec![CallFuncNode("input", vec![])])];
            return all;
        } = CallFuncNode("declvar", vec![Str("i"), Str("b"), CallFuncNode("parse_int", vec![CallFuncNode("input", vec![])])]);
        expect_comma();
        parse_function {
            get_identifier() = "print";
            get_arguments() {
                expect_lparen();
                parse_expression_or_rparen() = parse_function {
                    get_identifier() = "add";
                    get_arguments() {
                        expect_lparen();
                        parse_expression_or_rparen() = VarNode("a");
                        expect_comma();
                        parse_expression_or_rparen() = VarNode("b");
                        expect_comma() = None => args = vec![VarNode("a"), VarNode("b")];
                        return args;
                    } = vec![CallFuncNode("add", vec![VarNode("a"), VarNode("b")])];
                } = CallFuncNode("print", vec![CallFuncNode("add", vec![VarNode("a"), VarNode("b")])]);
                expect_comma() = None => expect_rparen(); args = vec![CallFuncNode("add", vec![VarNode("a"), VarNode("b")])];
                return args;
            } = vec![CallFuncNode("add", vec![VarNode("a"), VarNode("b")])];
            return all;
        } = CallFuncNode("print", vec![CallFuncNode("add", vec![VarNode("a"), VarNode("b")])]);
        expect_comma() = None;
    } = SequenceNode(CallFuncNode("declvar", vec![Str("i"), Str("a"), Int(5)]), CallFuncNode("declvar", vec![Str("i"), Str("b"), CallFuncNode("parse_int", vec![CallFuncNode("input", vec![])])]), CallFuncNode("print", vec![CallFuncNode("add", vec![VarNode("a"), VarNode("b")])]));
    # End of result
     */
    Ok(SequenceNode { node: Node { node_type: 0 }, body: vec![] })
}

pub fn parse(code: &str) -> Result<SequenceNode, ParserError> {
    let tokens_result = to_tokens(code);

    if tokens_result.is_err() {
        let error = tokens_result.unwrap_err();
        return Err(ParserError { line: error.line, column: error.column, description: error.description });
    }

    let mut tokens = tokens_result.unwrap();
    parse_tokens(&mut tokens)
}
