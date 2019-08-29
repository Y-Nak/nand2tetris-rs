use super::ast::*;
use super::symbol_table::*;
use std::rc::Rc;

pub struct VmGen {
    sym_table: SymbolTable,
    vm_code: String,
    label_count: usize,
}

impl VmGen {
    pub fn new() -> Self {
        Self {
            sym_table: SymbolTable::new(),
            vm_code: String::new(),
            label_count: 0,
        }
    }

    pub fn gen(&mut self, ast: ClassDec) -> Result<&str, &'static str> {
        self.sym_table.clear();
        self.label_count = 0;
        for class_var in &ast.var_decs {
            let kind = match class_var.var_ty {
                ClassVarType::Static => SymKind::Static,
                ClassVarType::Field => SymKind::Field,
            };
            let ty = SymType::from_astty(&class_var.ty)?;
            for name in &class_var.names {
                self.sym_table.insert(name.clone(), ty.clone(), kind);
            }
        }

        let field_count = ast
            .var_decs
            .iter()
            .filter(|var| var.var_ty == ClassVarType::Field)
            .flat_map(|var| &var.names)
            .fold(0, |acc, _| acc + 1);
        for subroutine_dec in &ast.subroutine_decs {
            self.subroutine_dec(subroutine_dec, &ast.name, field_count)?;
        }
        Ok(&self.vm_code)
    }
    fn subroutine_dec(
        &mut self,
        dec: &SubRoutineDec,
        class_name: &Rc<String>,
        field_count: usize,
    ) -> Result<(), &'static str> {
        self.sym_table.next_scope();
        if dec.kind == SubRoutineKind::Method {
            self.sym_table.insert(
                Rc::new("this".to_string()),
                SymType::Class(class_name.clone()),
                SymKind::Argument,
            );
        }

        for (ty, name) in &dec.args {
            let ty = SymType::from_astty(ty)?;
            self.sym_table
                .insert(name.clone(), ty.clone(), SymKind::Argument);
        }

        self.subroutine_body(&dec.body, class_name, &dec.name, dec.kind, field_count)
    }

    fn subroutine_body(
        &mut self,
        body: &SubRoutineBody,
        class_name: &Rc<String>,
        fn_name: &Rc<String>,
        kind: SubRoutineKind,
        field_count: usize,
    ) -> Result<(), &'static str> {
        let mut locals_count = 0;
        for var in &body.var_decs {
            let ty = SymType::from_astty(&var.ty)?;
            for name in &var.names {
                self.sym_table
                    .insert(name.clone(), ty.clone(), SymKind::Var);
                locals_count += 1;
            }
        }

        self.vm_code
            .push_str(&format! {"function {}.{} {}\n", class_name, fn_name, locals_count});
        match kind {
            SubRoutineKind::Constructor => {
                self.push("constant", field_count);
                self.vm_code.push_str("call Memory.alloc 1\n");
                self.pop("pointer", 0);
            }
            SubRoutineKind::Method => {
                self.push("argument", 0);
                self.pop("pointer", 0);
            }
            SubRoutineKind::Function => {}
        }

        for stmt in &body.stmts {
            self.stmt(stmt, class_name)?;
        }

        Ok(())
    }

    fn stmt(&mut self, stmt: &Stmt, class_name: &Rc<String>) -> Result<(), &'static str> {
        match stmt {
            Stmt::Let { name, idx, expr } => {
                let var = self.sym_table.get(name).ok_or("Undefined variable")?;
                let reg_name = var.reg_name();
                let id = var.id();
                self.expr(expr, class_name)?;

                if let Some(idx) = idx {
                    self.expr(idx, class_name)?;
                    self.push(reg_name, id);
                    self.binop(Binop::Plus);
                    self.pop("pointer", 1);
                    self.pop("that", 0);
                } else {
                    self.pop(reg_name, id);
                }
            }
            Stmt::If { test, then, else_ } => {
                let then_label = self.gen_label();
                let end_label = self.gen_label();
                self.expr(test, class_name)?;
                self.if_goto(&then_label);
                for stmt in else_.as_ref().unwrap_or(&Vec::new()) {
                    self.stmt(stmt, class_name)?;
                }
                self.goto(&end_label);
                self.label(&then_label);
                for stmt in then {
                    self.stmt(stmt, class_name)?;
                }
                self.label(&end_label);
            }
            Stmt::While { test, body } => {
                let stmt_label = self.gen_label();
                let loop_label = self.gen_label();
                let end_label = self.gen_label();
                self.label(&loop_label);
                self.expr(test, class_name)?;
                self.if_goto(&stmt_label);
                self.goto(&end_label);
                self.label(&stmt_label);
                for stmt in body {
                    self.stmt(stmt, class_name)?;
                }
                self.goto(&loop_label);
                self.label(&end_label);
            }
            Stmt::Do(call) => {
                self.call(call, class_name)?;
                self.pop("temp", 0);
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.expr(expr, class_name)?;
                } else {
                    self.push("constant", 0);
                }
                self.vm_code.push_str("return\n");
            }
        }
        Ok(())
    }

    fn expr(&mut self, expr: &Expr, class_name: &Rc<String>) -> Result<(), &'static str> {
        self.term(&expr.lhs, class_name)?;
        if let Some((op, ref term)) = expr.cdr {
            self.term(term, class_name)?;
            self.binop(op);
        }
        Ok(())
    }

    fn term(&mut self, term: &Term, class_name: &Rc<String>) -> Result<(), &'static str> {
        match term {
            Term::IntegerConstant(num) => self.push("constant", *num as usize),
            Term::StringConstant(s) => self.string_constant(s),
            Term::KeywordConstant(kwd) => match kwd {
                KeywordConstant::True => {
                    self.push("constant", 0);
                    self.unop(Unop::BitNot);
                }
                KeywordConstant::False | KeywordConstant::Null => self.push("constant", 0),
                KeywordConstant::This => self.push("pointer", 0),
            },
            Term::ValName(name) => {
                let entry = self.sym_table.get(name).ok_or("Undefined variable")?;
                let reg_name = entry.reg_name();
                let id = entry.id();
                self.push(reg_name, id);
            }
            Term::WithUnary(unary, term) => {
                self.term(term, class_name)?;
                self.unop(*unary);
            }
            Term::WithIdx(name, expr) => {
                self.expr(expr, class_name)?;
                let entry = self.sym_table.get(name).ok_or("Undefined variable")?;
                let reg_name = entry.reg_name();
                let id = entry.id();
                self.push(reg_name, id);
                self.binop(Binop::Plus);
                self.pop("pointer", 1);
                self.push("that", 0);
            }
            Term::SubRoutineCall(subroutine_call) => {
                self.call(subroutine_call, class_name)?;
            }
            Term::Expr(expr) => {
                self.expr(expr, class_name)?;
            }
        }
        Ok(())
    }

    fn push(&mut self, name: &str, index: usize) {
        self.vm_code
            .push_str(&format! {"push {} {}\n", name, index});
    }

    fn pop(&mut self, name: &str, index: usize) {
        self.vm_code.push_str(&format! {"pop {} {}\n", name, index});
    }

    fn call(&mut self, call: &SubRoutineCall, class_name: &Rc<String>) -> Result<(), &'static str> {
        let mut arg_num = call.args.len();
        let mangled_name = if let Some(obj_name) = &call.obj_name {
            if let Some(sym) = self.sym_table.get(&obj_name) {
                let reg_name = sym.reg_name();
                let id = sym.id();
                let class_name = sym.class_name()?;
                self.push(reg_name, id);
                arg_num += 1;
                class_name
            } else {
                obj_name.clone()
            }
        } else {
            self.push("pointer", 0);
            arg_num += 1;
            class_name.clone()
        };

        for expr in &call.args {
            self.expr(expr, &class_name)?;
        }

        self.vm_code
            .push_str(&format! {"call {}.{} {}\n", mangled_name, call.routine_name, arg_num});
        Ok(())
    }

    fn if_goto(&mut self, label: &str) {
        self.vm_code.push_str(&format! {"if-goto {}\n", label});
    }

    fn goto(&mut self, label: &str) {
        self.vm_code.push_str(&format! {"goto {}\n", label});
    }

    fn label(&mut self, label: &str) {
        self.vm_code.push_str(&format! {"label {}\n", label});
    }

    fn string_constant(&mut self, s: &str) {
        self.push("constant", s.len());
        self.vm_code.push_str("call String.new 1\n");
        for c in s.chars() {
            self.push("constant", c as usize);
            self.vm_code.push_str("call String.appendChar 2\n");
        }
    }

    fn binop(&mut self, op: Binop) {
        match op {
            Binop::Plus => self.vm_code.push_str("add\n"),
            Binop::Minus => self.vm_code.push_str("sub\n"),
            Binop::Mul => self.vm_code.push_str("call Math.multiply 2\n"),
            Binop::Div => self.vm_code.push_str("call Math.divide 2\n"),
            Binop::And => self.vm_code.push_str("and\n"),
            Binop::Or => self.vm_code.push_str("or\n"),
            Binop::Lt => self.vm_code.push_str("lt\n"),
            Binop::Gt => self.vm_code.push_str("gt\n"),
            Binop::Equal => self.vm_code.push_str("eq\n"),
        }
    }

    fn unop(&mut self, op: Unop) {
        match op {
            Unop::Minus => self.vm_code.push_str("neg\n"),
            Unop::BitNot => self.vm_code.push_str("not\n"),
        }
    }

    fn gen_label(&mut self) -> String {
        let label = format! {"jmp_label_{}", self.label_count};
        self.label_count += 1;
        label
    }
}
