use std::collections::HashMap;

use super::parser::*;

const MAX_FREE_RAM_ADDR: u16 = 16384;

pub struct SymbolTable<'a> {
    table: HashMap<&'a str, u16>,
    next_addr: u16,
}

impl<'a> SymbolTable<'a> {
    pub fn new(v: &'a Vec<Op>) -> Self {
        let mut table = HashMap::new();
        table.insert("SP", 0);
        table.insert("LCL", 1);
        table.insert("ARG", 2);
        table.insert("THIS", 3);
        table.insert("THAT", 4);
        table.insert("R0", 0);
        table.insert("R1", 1);
        table.insert("R2", 2);
        table.insert("R3", 3);
        table.insert("R4", 4);
        table.insert("R5", 5);
        table.insert("R6", 6);
        table.insert("R7", 7);
        table.insert("R8", 8);
        table.insert("R9", 9);
        table.insert("R10", 10);
        table.insert("R11", 11);
        table.insert("R12", 12);
        table.insert("R13", 13);
        table.insert("R14", 14);
        table.insert("R15", 15);
        table.insert("SCREEN", 16384);
        table.insert("KBD", 24576);

        let mut line = 0;
        for op in v {
            match op {
                Op::Addr(_) | Op::Comp(_) => {
                    line += 1;
                    continue;
                }
                Op::Label(ref s) => {
                    table.insert(s, line);
                }
            }
        }
        Self {
            table,
            next_addr: 16,
        }
    }

    pub fn get_or_insert(&mut self, s: &'a str) -> u16 {
        match self.table.get(s) {
            Some(&addr) => addr,
            None => {
                if self.next_addr == MAX_FREE_RAM_ADDR {
                    panic!("RAM Address overflow");
                }
                self.table.insert(s, self.next_addr);
                self.next_addr += 1;
                self.table[s]
            }
        }
    }
}
