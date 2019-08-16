use std::io::BufRead;
use std::str::FromStr;

use self::Command::*;
use self::{Op::*, Segment::*};

pub enum Op {
    Add,
    Sub,
    Neg,
    Eq_,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

pub enum Command {
    Arithmetic(Op),
    Push(Segment, u16),
    Pop(Segment, u16),
    Label(String),
    Goto(String),
    IfGoto(String),
    Function(String, u16),
    Call(String, u16),
    Return,
}

pub enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

pub fn parse_vm(strm: impl BufRead) -> Result<Vec<Command>, String> {
    let mut ret = Vec::new();
    for (line_num, line) in strm.lines().enumerate() {
        let line = line.map_err(|e| format! {"Error: line {}; {:?}", line_num + 1, e})?;
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        let command =
            parse_line(&line).map_err(|e| format! {"Error: line {}; {}", line_num + 1, e})?;
        ret.push(command);
    }
    Ok(ret)
}

fn parse_line(line: &str) -> Result<Command, &'static str> {
    let mut words = line.split_whitespace();
    let first = words.next().ok_or("Invalid VM code")?;
    let command = match first {
        "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
            Ok(Arithmetic(Op::from_str(first)?))
        }
        "push" => {
            let segment: Segment = next_as(&mut words)?;
            let offset: u16 = next_as(&mut words)?;
            Ok(Push(segment, offset))
        }
        "pop" => {
            let segment: Segment = next_as(&mut words)?;
            match segment {
                Constant => return Err("Invalid VM code"),
                _ => {}
            }
            let offset: u16 = next_as(&mut words)?;
            Ok(Pop(segment, offset))
        }
        "label" => {
            let label: String = next_as(&mut words)?;
            Ok(Label(label))
        }
        "goto" => {
            let label: String = next_as(&mut words)?;
            Ok(Goto(label))
        }
        "if-goto" => {
            let label: String = next_as(&mut words)?;
            Ok(IfGoto(label))
        }
        "function" => {
            let label: String = next_as(&mut words)?;
            let arg_num: u16 = next_as(&mut words)?;
            Ok(Function(label, arg_num))
        }
        "call" => {
            let label: String = next_as(&mut words)?;
            let arg_num: u16 = next_as(&mut words)?;
            Ok(Call(label, arg_num))
        }
        "return" => Ok(Return),
        _ => Err("Invalid VM code"),
    };

    match words.next() {
        Some("//") | None => command,
        _ => Err("Invalid VM code"),
    }
}

fn next_as<'a, T, U>(mut iter: U) -> Result<T, &'static str>
where
    T: FromStr,
    U: Iterator<Item = &'a str>,
{
    match iter.next() {
        Some(s) => s.parse().map_err(|_| "Invalid VM code"),
        None => Err("Invalid VM code"),
    }
}

impl FromStr for Op {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(Add),
            "sub" => Ok(Sub),
            "neg" => Ok(Neg),
            "eq" => Ok(Eq_),
            "gt" => Ok(Gt),
            "lt" => Ok(Lt),
            "and" => Ok(And),
            "or" => Ok(Or),
            "not" => Ok(Not),
            _ => unreachable! {},
        }
    }
}

impl FromStr for Segment {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "argument" => Ok(Segment::Argument),
            "local" => Ok(Segment::Local),
            "static" => Ok(Segment::Static),
            "constant" => Ok(Segment::Constant),
            "this" => Ok(Segment::This),
            "that" => Ok(Segment::That),
            "pointer" => Ok(Segment::Pointer),
            "temp" => Ok(Segment::Temp),
            _ => Err("Invalid VM code"),
        }
    }
}
