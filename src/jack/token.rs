use std::rc::Rc;

use Keyword::*;

#[derive(PartialEq, Eq, Debug)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Symbol {
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Dot,
    Comma,
    SemiColon,
    Plus,
    Minus,
    Star,
    Slush,
    And,
    Or,
    LAngle,
    RAngle,
    Equal,
    Tilde,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    IntegerConstant(u16),
    StringConstant(Rc<String>),
    Ident(Rc<String>),
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "class" => Some(Class),
            "constructor" => Some(Constructor),
            "function" => Some(Function),
            "method" => Some(Method),
            "field" => Some(Field),
            "static" => Some(Static),
            "var" => Some(Var),
            "int" => Some(Int),
            "char" => Some(Char),
            "boolean" => Some(Boolean),
            "void" => Some(Void),
            "true" => Some(True),
            "false" => Some(False),
            "null" => Some(Null),
            "this" => Some(This),
            "let" => Some(Let),
            "do" => Some(Do),
            "if" => Some(If),
            "else" => Some(Else),
            "while" => Some(While),
            "return" => Some(Return),
            _ => None,
        }
    }
}
