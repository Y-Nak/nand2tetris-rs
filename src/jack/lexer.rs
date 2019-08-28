use std::borrow::Borrow;
use std::rc::Rc;

use super::token::{Keyword, Symbol::*, Token};

pub fn tokenize<T, U>(strm: T) -> Result<Vec<Token>, String>
where
    T: Iterator<Item = U>,
    U: Borrow<char>,
{
    let mut ret = Vec::new();
    let mut strm = strm.peekable();
    let mut line_num = 1;
    while let Some(c) = strm.next() {
        match c.borrow() {
            '\n' => {
                line_num += 1;
            }
            '"' => {
                let mut s = String::new();
                loop {
                    if let Some(c) = strm.peek() {
                        if *c.borrow() == '"' {
                            strm.next().unwrap();
                            break;
                        } else if *c.borrow() == '\n' {
                            return Err(
                                format! {"Error: line {}; Can't use line break in string literal", line_num},
                            );
                        } else {
                            let c = strm.next().unwrap();
                            s.push(*c.borrow());
                        }
                    }
                    if let None = strm.peek() {
                        return Err(format! {"Error: line {}; Unclosed delimiter", line_num});
                    }
                }
                ret.push(Token::StringConstant(Rc::new(s)));
            }
            '{' => ret.push(Token::Symbol(LBrace)),
            '}' => ret.push(Token::Symbol(RBrace)),
            '(' => ret.push(Token::Symbol(LParen)),
            ')' => ret.push(Token::Symbol(RParen)),
            '[' => ret.push(Token::Symbol(LBracket)),
            ']' => ret.push(Token::Symbol(RBracket)),
            '.' => ret.push(Token::Symbol(Dot)),
            ',' => ret.push(Token::Symbol(Comma)),
            ';' => ret.push(Token::Symbol(SemiColon)),
            '+' => ret.push(Token::Symbol(Plus)),
            '-' => ret.push(Token::Symbol(Minus)),
            '*' => ret.push(Token::Symbol(Star)),
            '&' => ret.push(Token::Symbol(And)),
            '|' => ret.push(Token::Symbol(Or)),
            '<' => ret.push(Token::Symbol(LAngle)),
            '>' => ret.push(Token::Symbol(RAngle)),
            '=' => ret.push(Token::Symbol(Equal)),
            '~' => ret.push(Token::Symbol(Tilde)),
            '/' => {
                if let Some(c) = strm.peek() {
                    let c = *c.borrow();
                    if c == '/' {
                        skip_line(&mut strm);
                        line_num += 1;
                    } else if c == '*' {
                        line_num += skip_multiline_comment(&mut strm)?;
                    } else {
                        ret.push(Token::Symbol(Slush))
                    }
                }
            }

            c if c.is_ascii_alphabetic() => {
                let mut s = String::new();
                s.push(*c.borrow());
                loop {
                    if let Some(c) = strm.peek() {
                        if c.borrow().is_ascii_alphabetic()
                            || c.borrow().is_digit(10)
                            || *c.borrow() == '_'
                        {
                            s.push(*c.borrow());
                            strm.next();
                            continue;
                        }
                    }
                    break;
                }
                if let Some(keyword) = Keyword::from_str(&s) {
                    ret.push(Token::Keyword(keyword));
                } else {
                    ret.push(Token::Ident(Rc::new(s)));
                }
            }
            c if c.is_digit(10) => {
                let mut num = c.to_digit(10).unwrap() as u16;
                loop {
                    if let Some(c) = strm.peek() {
                        let c = *c.borrow();
                        if c.is_digit(10) {
                            strm.next().unwrap();
                            num *= 10;
                            num += c.borrow().to_digit(10).unwrap() as u16;
                        } else {
                            break;
                        }
                    }
                }
                ret.push(Token::IntegerConstant(num));
            }
            c if c.is_ascii_whitespace() => {}
            _ => {
                return Err(format! {"Error: line {}; Unexpected Token", line_num});
            }
        }
    }

    Ok(ret)
}

fn skip_line<T, U>(strm: &mut T)
where
    T: Iterator<Item = U>,
    U: Borrow<char>,
{
    while let Some(c) = strm.next() {
        if *c.borrow() == '\n' {
            return;
        }
    }
}

fn skip_multiline_comment<T, U>(strm: &mut T) -> Result<usize, String>
where
    T: Iterator<Item = U>,
    U: Borrow<char>,
{
    let mut line_count = 0;
    while let Some(c) = strm.next() {
        if *c.borrow() == '*' {
            if let Some(c) = strm.next() {
                if *c.borrow() == '/' {
                    return Ok(line_count);
                }
            }
        }
        if *c.borrow() == '\n' {
            line_count += 1;
        }
    }
    return Err("Multi line Comment must be closed".to_string());
}
