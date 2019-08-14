use std::io::BufRead;

use super::parser::*;
use super::symbol::SymbolTable;

const MAXIMUM_ADDR: u16 = 32767;

pub fn gen_code(strm: impl BufRead) -> Result<Vec<String>, String> {
    let ops = parse_asm(strm)?;
    let mut sym_table = SymbolTable::new(&ops);

    let mut ret = Vec::new();
    for op in &ops {
        match op {
            Op::Addr(addr) => {
                let addr = match addr {
                    Address::Symbol(label) => sym_table.get_or_insert(label),
                    Address::Immediate(num) => *num,
                };
                ret.push(gen_addressing(addr));
            }
            Op::Comp(Comp { dest, op, jmp }) => {
                ret.push(gen_comp(dest, op, jmp));
            }
            Op::Label(_) => {}
        }
    }
    Ok(ret)
}

fn gen_addressing(addr: u16) -> String {
    if addr > MAXIMUM_ADDR {
        panic!("Memory overflow, address is too large.");
    }

    format!("0{:015b}", addr)
}

fn gen_comp(dest: &Dest, op: &CompOp, jmp: &Jmp) -> String {
    let dest = match dest {
        Dest::Null => 0b000,
        Dest::M => 0b001,
        Dest::D => 0b010,
        Dest::MD => 0b011,
        Dest::A => 0b100,
        Dest::AM => 0b101,
        Dest::AD => 0b110,
        Dest::AMD => 0b111,
    };

    let (op, use_m) = match op {
        &CompOp::Zero => (0b101010, false),
        &CompOp::One => (0b111111, false),
        &CompOp::NegOne => (0b111010, false),
        &CompOp::D => (0b001100, false),
        &CompOp::AM(use_m) => (0b110000, use_m),
        &CompOp::NotD => (0b001101, false),
        &CompOp::NotAM(use_m) => (0b110001, use_m),
        &CompOp::NegD => (0b001111, false),
        &CompOp::NegAM(use_m) => (0b110011, use_m),
        &CompOp::IncD => (0b011111, false),
        &CompOp::IncAM(use_m) => (0b110111, use_m),
        &CompOp::DecD => (0b001110, false),
        &CompOp::DecAM(use_m) => (0b110010, use_m),
        &CompOp::DPlusAM(use_m) => (0b000010, use_m),
        &CompOp::DMinusAM(use_m) => (0b010011, use_m),
        &CompOp::AMMinusD(use_m) => (0b000111, use_m),
        &CompOp::DAndAM(use_m) => (0b000000, use_m),
        &CompOp::DOrAM(use_m) => (0b010101, use_m),
    };

    let use_m = if use_m { 0b1 } else { 0b0 };
    let jmp = match jmp {
        Jmp::Null => 0b000,
        Jmp::Jgt => 0b001,
        Jmp::Jeq => 0b010,
        Jmp::Jge => 0b011,
        Jmp::Jlt => 0b100,
        Jmp::Jne => 0b101,
        Jmp::Jle => 0b110,
        Jmp::Jmp => 0b111,
    };

    format!("111{:b}{:06b}{:03b}{:03b}", use_m, op, dest, jmp)
}
