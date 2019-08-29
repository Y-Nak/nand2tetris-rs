use std::rc::Rc;

pub struct ClassDec {
    pub name: Rc<String>,
    pub var_decs: Vec<ClassVarDec>,
    pub subroutine_decs: Vec<SubRoutineDec>,
}

pub struct ClassVarDec {
    pub var_ty: ClassVarType,
    pub ty: Type,
    pub names: Vec<Rc<String>>,
}

pub struct SubRoutineDec {
    pub name: Rc<String>,
    pub kind: SubRoutineKind,
    pub ret: Type,
    pub args: Vec<(Type, Rc<String>)>,
    pub body: SubRoutineBody,
}

pub struct SubRoutineBody {
    pub var_decs: Vec<VarDec>,
    pub stmts: Vec<Stmt>,
}

pub struct VarDec {
    pub names: Vec<Rc<String>>,
    pub ty: Type,
}

pub enum Stmt {
    Let {
        name: Rc<String>,
        idx: Option<Expr>,
        expr: Expr,
    },
    If {
        test: Expr,
        then: Vec<Stmt>,
        else_: Option<Vec<Stmt>>,
    },
    While {
        test: Expr,
        body: Vec<Stmt>,
    },
    Do(SubRoutineCall),
    Return(Option<Expr>),
}

pub struct Expr {
    pub lhs: Term,
    pub cdr: Option<(Binop, Box<Term>)>,
}

pub enum Term {
    IntegerConstant(u16),
    StringConstant(Rc<String>),
    KeywordConstant(KeywordConstant),
    ValName(Rc<String>),
    WithUnary(Unop, Box<Term>),
    WithIdx(Rc<String>, Box<Expr>),
    SubRoutineCall(SubRoutineCall),
    Expr(Box<Expr>),
}

#[derive(PartialEq, Eq)]
pub enum ClassVarType {
    Static,
    Field,
}

pub enum Type {
    Class(Rc<String>),
    Int,
    Char,
    Boolean,
    Void,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SubRoutineKind {
    Constructor,
    Function,
    Method,
}

pub struct SubRoutineCall {
    pub obj_name: Option<Rc<String>>,
    pub routine_name: Rc<String>,
    pub args: Vec<Expr>,
}

#[derive(Clone, Copy)]
pub enum Binop {
    Plus,
    Minus,
    Mul,
    Div,
    And,
    Or,
    Gt,
    Lt,
    Equal,
}

#[derive(Clone, Copy)]
pub enum Unop {
    Minus,
    BitNot,
}

pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}

macro_rules! symbol {
    ($s:expr) => {
        concat!("<symbol> ", $s, " </symbol>\n")
    };
}

macro_rules! keyword {
    ($s:expr) => {
        concat!("<keyword> ", $s, " </keyword>\n");
    };
}

macro_rules! ident {
    ($s:expr) => {
        format!("<identifier> {} </identifier>\n", $s)
    };
}

impl ClassDec {
    pub fn new(
        name: Rc<String>,
        var_decs: Vec<ClassVarDec>,
        subroutine_decs: Vec<SubRoutineDec>,
    ) -> Self {
        Self {
            name,
            var_decs,
            subroutine_decs,
        }
    }

    pub fn to_xml(&self) -> String {
        let mut ret = String::new();
        ret.push_str("<class>\n");
        ret.push_str(keyword! {"class"});
        ret.push_str(&ident! {self.name});
        ret.push_str(symbol!("{"));
        if !self.var_decs.is_empty() {
            for var_dec in &self.var_decs {
                var_dec.write(&mut ret);
            }
        }
        for routine_dec in &self.subroutine_decs {
            &routine_dec.write(&mut ret);
        }
        ret.push_str(symbol! {"}"});
        ret.push_str("</class>\n");
        ret
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ClassVarDec {
    pub fn new(var_ty: ClassVarType, ty: Type, names: Vec<Rc<String>>) -> Self {
        Self { var_ty, ty, names }
    }

    fn write(&self, s: &mut String) {
        s.push_str("<classVarDec>\n");
        match self.var_ty {
            ClassVarType::Field => s.push_str(keyword! {"field"}),
            ClassVarType::Static => s.push_str(keyword! {"static"}),
        }
        self.ty.write(s);
        s.push_str(&ident! {self.names[0]});
        for name in self.names.iter().skip(1) {
            s.push_str(symbol! {","});
            s.push_str(&ident! {name});
        }
        s.push_str(symbol! {";"});
        s.push_str("</classVarDec>\n");
    }
}

impl SubRoutineDec {
    pub fn new(
        name: Rc<String>,
        kind: SubRoutineKind,
        ret: Type,
        args: Vec<(Type, Rc<String>)>,
        body: SubRoutineBody,
    ) -> Self {
        Self {
            name,
            kind,
            ret,
            args,
            body,
        }
    }

    fn write(&self, s: &mut String) {
        s.push_str("<subroutineDec>\n");
        match self.kind {
            SubRoutineKind::Constructor => s.push_str(keyword! {"constructor"}),
            SubRoutineKind::Function => s.push_str(keyword! { "function"}),
            SubRoutineKind::Method => s.push_str(keyword! {"method"}),
        }
        &self.ret.write(s);
        s.push_str(&ident! {self.name});
        s.push_str(symbol! {"("});
        s.push_str("<parameterList>\n");
        for (i, (ty, name)) in self.args.iter().enumerate() {
            if i > 0 {
                s.push_str(symbol! {","});
            }
            ty.write(s);
            s.push_str(&ident! {name});
        }
        s.push_str("</parameterList>\n");
        s.push_str(symbol! {")"});
        self.body.write(s);
        s.push_str("</subroutineDec>\n");
    }
}

impl SubRoutineBody {
    pub fn new(var_decs: Vec<VarDec>, stmts: Vec<Stmt>) -> Self {
        Self { var_decs, stmts }
    }

    fn write(&self, s: &mut String) {
        s.push_str("<subroutineBody>\n");
        s.push_str(symbol! {"{"});
        for var_dec in &self.var_decs {
            s.push_str("<varDec>\n");
            var_dec.write(s);
            s.push_str("</varDec>\n");
        }
        s.push_str("<statements>\n");
        for stmt in &self.stmts {
            stmt.write(s);
        }
        s.push_str("</statements>\n");
        s.push_str(symbol! {"}"});
        s.push_str("</subroutineBody>\n");
    }
}

impl VarDec {
    pub fn new(names: Vec<Rc<String>>, ty: Type) -> Self {
        Self { names, ty }
    }

    fn write(&self, s: &mut String) {
        s.push_str(keyword! {"var"});
        self.ty.write(s);
        s.push_str(&ident!(self.names[0]));
        for name in self.names.iter().skip(1) {
            s.push_str(symbol!(","));
            s.push_str(&ident! {name});
        }
        s.push_str(symbol!(";"));
    }
}

impl Stmt {
    fn write(&self, s: &mut String) {
        match self {
            Stmt::Let { name, idx, expr } => {
                s.push_str("<letStatement>\n");
                s.push_str(keyword! {"let"});
                s.push_str(&ident! {name});
                if let Some(expr) = idx {
                    s.push_str(symbol! {"["});
                    expr.write(s);
                    s.push_str(symbol! {"]"});
                }
                s.push_str(symbol! {"="});
                expr.write(s);
                s.push_str(symbol! {";"});
                s.push_str("</letStatement>\n");
            }
            Stmt::If { test, then, else_ } => {
                s.push_str("<ifStatement>\n");
                s.push_str(keyword! {"if"});
                s.push_str(symbol! {"("});
                test.write(s);
                s.push_str(symbol! {")"});
                s.push_str(symbol! {"{"});
                s.push_str("<statements>\n");
                for stmt in then {
                    stmt.write(s);
                }
                s.push_str("</statements>\n");
                s.push_str(symbol! {"}"});
                if let Some(else_) = else_ {
                    s.push_str(keyword!("else"));
                    s.push_str(symbol! {"{"});
                    s.push_str("<statements>\n");
                    for stmt in else_ {
                        stmt.write(s);
                    }
                    s.push_str("</statements>\n");
                    s.push_str(symbol! {"}"});
                }
                s.push_str("</ifStatement>\n");
            }
            Stmt::While { test, body } => {
                s.push_str("<whileStatement>\n");
                s.push_str(keyword! {"while"});
                s.push_str(symbol! {"("});
                test.write(s);
                s.push_str(symbol! {")"});
                s.push_str(symbol! {"{"});
                s.push_str("<statements>\n");
                for stmt in body {
                    stmt.write(s);
                }
                s.push_str("</statements>\n");
                s.push_str(symbol! {"}"});
                s.push_str("</whileStatement>\n");
            }
            Stmt::Do(subroutine_call) => {
                s.push_str("<doStatement>\n");
                s.push_str(keyword! {"do"});
                subroutine_call.write(s);
                s.push_str(symbol! {";"});
                s.push_str("</doStatement>\n");
            }
            Stmt::Return(expr) => {
                s.push_str("<returnStatement>\n");
                s.push_str(keyword! {"return"});
                if let Some(expr) = expr {
                    expr.write(s);
                }
                s.push_str(symbol! {";"});
                s.push_str("</returnStatement>\n");
            }
        }
    }
}

impl Expr {
    pub fn new(lhs: Term, cdr: Option<(Binop, Box<Term>)>) -> Self {
        Self { lhs, cdr }
    }

    fn write(&self, s: &mut String) {
        s.push_str("<expression>\n");
        self.lhs.write(s);
        if let Some((binop, term)) = &self.cdr {
            match binop {
                Binop::Plus => s.push_str(symbol! {"+"}),
                Binop::Minus => s.push_str(symbol! {"-"}),
                Binop::Mul => s.push_str(symbol! {"*"}),
                Binop::Div => s.push_str(symbol! {"/"}),
                Binop::And => s.push_str(symbol! {"&amp;"}),
                Binop::Or => s.push_str(symbol! {"|"}),
                Binop::Gt => s.push_str(symbol! {"&gt;"}),
                Binop::Lt => s.push_str(symbol! {"&lt;"}),
                Binop::Equal => s.push_str(symbol! {"="}),
            }
            term.write(s);
        }
        s.push_str("</expression>\n");
    }
}

impl Term {
    fn write(&self, s: &mut String) {
        s.push_str("<term>\n");
        match self {
            Term::IntegerConstant(num) => {
                s.push_str(&format!("<integerConstant> {} </integerConstant>\n", num));
            }
            Term::StringConstant(cnstr) => {
                s.push_str(&format!("<stringConstant> {} </stringConstant>\n", cnstr));
            }
            Term::KeywordConstant(kwd) => match kwd {
                KeywordConstant::True => s.push_str(keyword! {"true"}),
                KeywordConstant::False => s.push_str(keyword! {"false"}),
                KeywordConstant::Null => s.push_str(keyword! {"null"}),
                KeywordConstant::This => s.push_str(keyword! {"this"}),
            },
            Term::ValName(name) => {
                s.push_str(&ident! {name});
            }
            Term::WithUnary(unop, term) => {
                match unop {
                    Unop::Minus => s.push_str(symbol!("-")),
                    Unop::BitNot => s.push_str(symbol!("~")),
                }
                term.write(s);
            }
            Term::WithIdx(name, expr) => {
                s.push_str(&ident! {name});
                s.push_str(symbol!("["));
                expr.write(s);
                s.push_str(symbol!("]"));
            }
            Term::SubRoutineCall(call) => {
                call.write(s);
            }
            Term::Expr(expr) => {
                s.push_str(symbol! {"("});
                expr.write(s);
                s.push_str(symbol! {")"});
            }
        }
        s.push_str("</term>\n");
    }
}

impl SubRoutineCall {
    pub fn new(obj_name: Option<Rc<String>>, routine_name: Rc<String>, args: Vec<Expr>) -> Self {
        Self {
            obj_name,
            routine_name,
            args,
        }
    }

    fn write(&self, s: &mut String) {
        if let Some(obj_name) = &self.obj_name {
            s.push_str(&ident!(obj_name));
            s.push_str(symbol!("."));
        }
        s.push_str(&ident! {self.routine_name});
        s.push_str(symbol! {"("});
        s.push_str("<expressionList>\n");
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                s.push_str(symbol! {","});
            }
            arg.write(s);
        }
        s.push_str("</expressionList>\n");
        s.push_str(symbol! {")"});
    }
}

impl Type {
    fn write(&self, s: &mut String) {
        match self {
            Type::Class(ref name) => s.push_str(&ident! {name}),
            Type::Int => s.push_str(keyword! {"int"}),
            Type::Char => s.push_str(keyword! {"char"}),
            Type::Boolean => s.push_str(keyword! {"boolean"}),
            Type::Void => s.push_str(keyword! {"void"}),
        }
    }
}
