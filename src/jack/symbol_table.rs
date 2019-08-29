use std::collections::HashMap;
use std::rc::Rc;

use super::ast::Type;

pub struct SymEntry {
    ty: SymType,
    kind: SymKind,
    index: usize,
}

pub struct SymbolTable {
    class_table: HashMap<Rc<String>, SymEntry>,
    subroutine_table: HashMap<Rc<String>, SymEntry>,
    static_count: usize,
    field_count: usize,
    arg_count: usize,
    var_count: usize,
}

#[derive(Clone)]
pub enum SymType {
    Class(Rc<String>),
    Int,
    Char,
    Boolean,
}

#[derive(Clone, Copy)]
pub enum SymKind {
    Static,
    Field,
    Argument,
    Var,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            class_table: HashMap::new(),
            subroutine_table: HashMap::new(),
            static_count: 0,
            field_count: 0,
            arg_count: 0,
            var_count: 0,
        }
    }

    pub fn insert(&mut self, name: Rc<String>, ty: SymType, kind: SymKind) {
        match kind {
            SymKind::Static => {
                let sym_info = SymEntry::new(ty, kind, self.static_count);
                self.class_table.insert(name, sym_info);
                self.static_count += 1;
            }
            SymKind::Field => {
                let sym_info = SymEntry::new(ty, kind, self.field_count);
                self.class_table.insert(name, sym_info);
                self.field_count += 1;
            }
            SymKind::Argument => {
                let sym_info = SymEntry::new(ty, kind, self.arg_count);
                self.subroutine_table.insert(name, sym_info);
                self.arg_count += 1;
            }
            SymKind::Var => {
                let sym_info = SymEntry::new(ty, kind, self.var_count);
                self.subroutine_table.insert(name, sym_info);
                self.var_count += 1;
            }
        }
    }

    pub fn next_scope(&mut self) {
        self.arg_count = 0;
        self.var_count = 0;
        self.subroutine_table.clear();
    }

    pub fn get(&self, name: &String) -> Option<&SymEntry> {
        let subroutine_symbol = self.subroutine_table.get(name);
        if subroutine_symbol.is_some() {
            subroutine_symbol
        } else {
            self.class_table.get(name)
        }
    }

    pub fn clear(&mut self) {
        self.class_table.clear();
        self.subroutine_table.clear();
        self.static_count = 0;
        self.field_count = 0;
        self.arg_count = 0;
        self.var_count = 0;
    }
}

impl SymEntry {
    fn new(ty: SymType, kind: SymKind, index: usize) -> Self {
        Self { ty, kind, index }
    }

    pub fn class_name(&self) -> Result<Rc<String>, &'static str> {
        match &self.ty {
            SymType::Class(name) => Ok(name.clone()),
            _ => Err("Error"),
        }
    }

    pub fn id(&self) -> usize {
        self.index
    }

    pub fn reg_name(&self) -> &'static str {
        match self.kind {
            SymKind::Static => "static",
            SymKind::Field => "this",
            SymKind::Argument => "argument",
            SymKind::Var => "local",
        }
    }
}

impl SymType {
    pub fn from_astty(ty: &Type) -> Result<Self, &'static str> {
        match ty {
            Type::Class(name) => Ok(SymType::Class(name.clone())),
            Type::Int => Ok(SymType::Int),
            Type::Char => Ok(SymType::Char),
            Type::Boolean => Ok(SymType::Boolean),
            Type::Void => Err("Error"),
        }
    }
}
