use std::io::BufRead;
use std::str;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(PartialEq, Eq, Debug)]
pub enum Dest {
    Null,
    M,
    D,
    MD,
    A,
    AM,
    AD,
    AMD,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CompOp {
    Zero,
    One,
    NegOne,
    D,
    AM(bool),
    NotD,
    NotAM(bool),
    NegD,
    NegAM(bool),
    IncD,
    IncAM(bool),
    DecD,
    DecAM(bool),
    DPlusAM(bool),
    DMinusAM(bool),
    AMMinusD(bool),
    DAndAM(bool),
    DOrAM(bool),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Jmp {
    Null,
    Jgt,
    Jeq,
    Jge,
    Jlt,
    Jne,
    Jle,
    Jmp,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Comp {
    pub dest: Dest,
    pub op: CompOp,
    pub jmp: Jmp,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Address {
    Symbol(String),
    Immediate(u16),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Op {
    Addr(Address),
    Comp(Comp),
    Label(String),
}

pub fn parse_asm<T>(strm: T) -> Result<Vec<Op>, String>
where
    T: BufRead,
{
    let mut ops = Vec::new();
    for (line_num, line) in strm.lines().enumerate() {
        let line_num = line_num + 1;
        let s = line.map_err(|e| format!("Error: line {}; {:?}", line_num, e))?;
        let s = &trim_str(&s);
        if s.is_empty() {
            continue;
        }
        ops.push(parse_line(&s).map_err(|e| format!("Error: line {}; {}", line_num, e))?);
    }
    Ok(ops)
}

fn parse_line(s: &str) -> Result<Op, &'static str> {
    lazy_static! {
        static ref OP_PAT: Regex =
            Regex::new(r"^(?:([AMD]+)=)?([AMD01\+\-!&\|]+);?(?:;(JGT|JEQ|JGE|JLT|JNE|JLE|JMP))?$")
                .unwrap();
        static ref ADDR_PAT: Regex = Regex::new(r"^@([[[:alnum:]]_.$:]+)").unwrap();
        static ref LABEL_PAT: Regex = Regex::new(r"\(([[[:alnum:]]_.$:]+)\)").unwrap();
    }

    let op_matches: Vec<_> = OP_PAT.captures_iter(s).collect();
    if 0 < op_matches.len() && op_matches.len() < 2 {
        let dest = op_matches[0].get(1).map(|s| s.as_str());
        let op = op_matches[0]
            .get(2)
            .map(|s| s.as_str())
            .ok_or("Invalid asm")?;
        let jmp = op_matches[0].get(3).map(|s| s.as_str());
        return parse_comp(dest, op, jmp);
    }

    let addr_matches: Vec<_> = ADDR_PAT.captures_iter(s).collect();
    if 0 < addr_matches.len() && addr_matches.len() < 2 {
        let addr = addr_matches[0].get(1).map(|s| s.as_str()).unwrap();
        return parse_addr(addr);
    }

    let label_matches: Vec<_> = LABEL_PAT.captures_iter(s).collect();
    if 0 < label_matches.len() && label_matches.len() < 2 {
        let label = label_matches[0].get(1).map(|s| s.as_str()).unwrap();
        return parse_label(label);
    }

    Err("Invalid asm")
}

fn trim_str(s: &str) -> String {
    let mut ret = String::new();
    let mut s = s.chars().peekable();
    while let Some(&c) = s.peek() {
        if c.is_whitespace() || c.is_whitespace() || c.is_control() {
            s.next();
            continue;
        }
        if c == '/' {
            s.next();
            if let Some('/') = s.peek() {
                break;
            }
            ret.push('/');
        }
        ret.push(c);
        s.next();
    }
    ret
}

fn parse_comp(dest: Option<&str>, op: &str, jmp: Option<&str>) -> Result<Op, &'static str> {
    let dest = if let Some(s) = dest {
        match s {
            "M" => Dest::M,
            "D" => Dest::D,
            "MD" | "DM" => Dest::MD,
            "A" => Dest::A,
            "AM" | "MA" => Dest::AM,
            "AD" | "DA" => Dest::AD,
            "AMD" | "ADM" | "DAM" | "DMA" | "MAD" | "MDA" => Dest::AMD,
            _ => return Err("Invalid asm"),
        }
    } else {
        Dest::Null
    };

    let use_m = op.contains("M");
    let op = match op.replace("M", "A").as_str() {
        "0" => CompOp::Zero,
        "1" => CompOp::One,
        "-1" => CompOp::NegOne,
        "D" => CompOp::D,
        "A" => CompOp::AM(use_m),
        "!D" => CompOp::NotD,
        "!A" => CompOp::NotAM(use_m),
        "-D" => CompOp::NegD,
        "-A" => CompOp::NegAM(use_m),
        "D+1" => CompOp::IncD,
        "A+1" => CompOp::IncAM(use_m),
        "D-1" => CompOp::DecD,
        "A-1" => CompOp::DecAM(use_m),
        "D+A" | "A+D" => CompOp::DPlusAM(use_m),
        "D-A" => CompOp::DMinusAM(use_m),
        "A-D" => CompOp::AMMinusD(use_m),
        "D&A" | "A&D" => CompOp::DAndAM(use_m),
        "D|A" | "A|D" => CompOp::DOrAM(use_m),
        _ => return Err("Invalid asm"),
    };

    let jmp = if let Some(s) = jmp {
        match s {
            "JGT" => Jmp::Jgt,
            "JEQ" => Jmp::Jeq,
            "JGE" => Jmp::Jge,
            "JLT" => Jmp::Jlt,
            "JNE" => Jmp::Jne,
            "JLE" => Jmp::Jle,
            "JMP" => Jmp::Jmp,
            _ => return Err("Invalid asm"),
        }
    } else {
        Jmp::Null
    };

    let comp = Comp { dest, op, jmp };
    Ok(Op::Comp(comp))
}

fn parse_addr(addr: &str) -> Result<Op, &'static str> {
    if addr.chars().all(|c| c.is_numeric()) {
        let num = addr.parse::<u16>().map_err(|_| "Invalid asm")?;
        Ok(Op::Addr(Address::Immediate(num)))
    } else {
        Ok(Op::Addr(Address::Symbol(addr.to_string())))
    }
}

fn parse_label(label: &str) -> Result<Op, &'static str> {
    Ok(Op::Label(label.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_addr() {
        let asm = "@$JMP.END\n@2143";
        let result = parse_asm(asm.as_bytes()).unwrap();
        assert_eq!(result[0], Op::Addr(Address::Symbol("$JMP.END".to_string())));
        assert_eq!(result[1], Op::Addr(Address::Immediate(2143)));
    }

    #[test]
    fn test_comp() {
        let asm = "AM=A+1;JMP\nD\nAMD=D+M;JLT\n";
        let result = parse_asm(asm.as_bytes()).unwrap();
        assert_eq!(
            result[0],
            Op::Comp(Comp {
                dest: Dest::AM,
                op: CompOp::IncAM(false),
                jmp: Jmp::Jmp,
            })
        );
        assert_eq!(
            result[1],
            Op::Comp(Comp {
                dest: Dest::Null,
                op: CompOp::D,
                jmp: Jmp::Null,
            })
        );
        assert_eq!(
            result[2],
            Op::Comp(Comp {
                dest: Dest::AMD,
                op: CompOp::DPlusAM(true),
                jmp: Jmp::Jlt,
            })
        );
    }

    #[test]
    fn test_label() {
        let asm = "(FOO)";
        let result = parse_asm(asm.as_bytes()).unwrap();
        assert_eq!(result[0], Op::Label("FOO".to_string()));

        let asm = "FOO";
        assert!(parse_asm(asm.as_bytes()).is_err());
    }
}
