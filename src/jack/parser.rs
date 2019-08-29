use std::iter::Peekable;
use std::rc::Rc;

use super::ast::*;
use super::token::{Keyword::*, Symbol::*, *};

pub struct Parser<T>
where
    T: Iterator<Item = Token>,
{
    tokens: Peekable<T>,
}

impl<T> Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub fn new(tokens: T) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<ClassDec, String> {
        self.class_dec()
    }

    fn class_dec(&mut self) -> Result<ClassDec, String> {
        self.eat_assert(Token::Keyword(Keyword::Class))?;
        let name = self.eat_ident()?;
        self.eat_assert(Token::Symbol(Symbol::LBrace))?;
        let var_decs = self.class_var_decs()?;
        let subroutine_decs = self.subroutine_decs()?;
        self.eat_assert(Token::Symbol(RBrace))?;
        Ok(ClassDec::new(name, var_decs, subroutine_decs))
    }

    fn class_var_decs(&mut self) -> Result<Vec<ClassVarDec>, String> {
        let mut ret = Vec::new();
        loop {
            let var_ty = match self.tokens.peek() {
                Some(Token::Keyword(Keyword::Static)) => ClassVarType::Static,
                Some(Token::Keyword(Keyword::Field)) => ClassVarType::Field,
                _ => return Ok(ret),
            };
            self.tokens.next();
            let ty = self.eat_type(false)?;
            let mut names = Vec::new();
            names.push(self.eat_ident()?);
            while let Some(Token::Symbol(Comma)) = self.tokens.peek() {
                self.eat_assert(Token::Symbol(Comma))?;
                names.push(self.eat_ident()?);
            }
            self.eat_assert(Token::Symbol(SemiColon))?;
            ret.push(ClassVarDec::new(var_ty, ty, names));
        }
    }

    fn subroutine_decs(&mut self) -> Result<Vec<SubRoutineDec>, String> {
        let mut ret = Vec::new();
        loop {
            let ty = match self.tokens.peek() {
                Some(Token::Keyword(Constructor)) => SubRoutineKind::Constructor,
                Some(Token::Keyword(Function)) => SubRoutineKind::Function,
                Some(Token::Keyword(Method)) => SubRoutineKind::Method,
                _ => return Ok(ret),
            };
            self.tokens.next();
            let ret_ty = self.eat_type(true)?;
            let name = self.eat_ident()?;
            self.eat_assert(Token::Symbol(LParen))?;
            let mut params = Vec::new();
            if let Ok(ty) = self.eat_type(false) {
                let name = self.eat_ident()?;
                params.push((ty, name));
                while let Some(Token::Symbol(Comma)) = self.tokens.peek() {
                    self.eat_assert(Token::Symbol(Comma))?;
                    let ty = self.eat_type(false)?;
                    let name = self.eat_ident()?;
                    params.push((ty, name));
                }
            }
            self.eat_assert(Token::Symbol(RParen))?;
            self.eat_assert(Token::Symbol(LBrace))?;
            let mut var_decs = Vec::new();
            while let Some(Token::Keyword(Var)) = self.tokens.peek() {
                self.eat_assert(Token::Keyword(Var))?;
                let ty = self.eat_type(false)?;
                let mut names = Vec::new();
                names.push(self.eat_ident()?);
                while let Some(Token::Symbol(Comma)) = self.tokens.peek() {
                    self.eat_assert(Token::Symbol(Comma))?;
                    names.push(self.eat_ident()?);
                }
                self.eat_assert(Token::Symbol(SemiColon))?;
                var_decs.push(VarDec::new(names, ty));
            }
            let stmts = self.stmts()?;
            let body = SubRoutineBody::new(var_decs, stmts);
            ret.push(SubRoutineDec::new(name, ty, ret_ty, params, body));
            self.eat_assert(Token::Symbol(RBrace))?;
        }
    }

    fn stmts(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            match self.tokens.peek() {
                Some(Token::Keyword(Let)) => stmts.push(self.let_stmt()?),
                Some(Token::Keyword(If)) => stmts.push(self.if_stmt()?),
                Some(Token::Keyword(While)) => stmts.push(self.while_stmt()?),
                Some(Token::Keyword(Do)) => stmts.push(self.do_stmt()?),
                Some(Token::Keyword(Return)) => stmts.push(self.return_stmt()?),
                _ => break,
            }
        }
        Ok(stmts)
    }

    fn let_stmt(&mut self) -> Result<Stmt, String> {
        self.eat_assert(Token::Keyword(Let))?;
        let name = self.eat_ident()?;
        let idx = if let Some(Token::Symbol(LBracket)) = self.tokens.peek() {
            self.eat_assert(Token::Symbol(LBracket))?;
            let expr = self.expr()?;
            self.eat_assert(Token::Symbol(RBracket))?;
            Some(expr)
        } else {
            None
        };
        self.eat_assert(Token::Symbol(Equal))?;
        let expr = self.expr()?;
        self.eat_assert(Token::Symbol(SemiColon))?;
        Ok(Stmt::Let { name, idx, expr })
    }

    fn if_stmt(&mut self) -> Result<Stmt, String> {
        self.eat_assert(Token::Keyword(If))?;
        self.eat_assert(Token::Symbol(LParen))?;
        let test = self.expr()?;
        self.eat_assert(Token::Symbol(RParen))?;
        self.eat_assert(Token::Symbol(LBrace))?;
        let then = self.stmts()?;
        self.eat_assert(Token::Symbol(RBrace))?;
        let else_ = if let Some(Token::Keyword(Else)) = self.tokens.peek() {
            self.eat_assert(Token::Keyword(Else))?;
            self.eat_assert(Token::Symbol(LBrace))?;
            let else_ = self.stmts()?;
            self.eat_assert(Token::Symbol(RBrace))?;
            Some(else_)
        } else {
            None
        };
        Ok(Stmt::If { test, then, else_ })
    }

    fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.eat_assert(Token::Keyword(While))?;
        self.eat_assert(Token::Symbol(LParen))?;
        let test = self.expr()?;
        self.eat_assert(Token::Symbol(RParen))?;
        self.eat_assert(Token::Symbol(LBrace))?;
        let body = self.stmts()?;
        self.eat_assert(Token::Symbol(RBrace))?;
        Ok(Stmt::While { test, body })
    }

    fn do_stmt(&mut self) -> Result<Stmt, String> {
        self.eat_assert(Token::Keyword(Do))?;
        let name = self.eat_ident()?;
        let do_stmt = Stmt::Do(self.subroutine_call(name)?);
        self.eat_assert(Token::Symbol(SemiColon))?;
        Ok(do_stmt)
    }

    fn return_stmt(&mut self) -> Result<Stmt, String> {
        self.eat_assert(Token::Keyword(Return))?;
        let expr = if let Some(Token::Symbol(SemiColon)) = self.tokens.peek() {
            self.eat_assert(Token::Symbol(SemiColon))?;
            None
        } else {
            let expr = self.expr()?;
            self.eat_assert(Token::Symbol(SemiColon))?;
            Some(expr)
        };
        Ok(Stmt::Return(expr))
    }

    fn expr(&mut self) -> Result<Expr, String> {
        let term = self.term()?;
        let cdr = match self.tokens.peek() {
            Some(Token::Symbol(Plus)) => {
                self.eat_assert(Token::Symbol(Plus))?;
                Some((Binop::Plus, Box::new(self.term()?)))
            }
            Some(Token::Symbol(Minus)) => {
                self.eat_assert(Token::Symbol(Minus))?;
                Some((Binop::Minus, Box::new(self.term()?)))
            }
            Some(Token::Symbol(Star)) => {
                self.eat_assert(Token::Symbol(Star))?;
                Some((Binop::Mul, Box::new(self.term()?)))
            }
            Some(Token::Symbol(Slush)) => {
                self.eat_assert(Token::Symbol(Slush))?;
                Some((Binop::Div, Box::new(self.term()?)))
            }
            Some(Token::Symbol(And)) => {
                self.eat_assert(Token::Symbol(And))?;
                Some((Binop::And, Box::new(self.term()?)))
            }
            Some(Token::Symbol(Or)) => {
                self.eat_assert(Token::Symbol(Or))?;
                Some((Binop::Or, Box::new(self.term()?)))
            }
            Some(Token::Symbol(LAngle)) => {
                self.eat_assert(Token::Symbol(LAngle))?;
                Some((Binop::Lt, Box::new(self.term()?)))
            }
            Some(Token::Symbol(RAngle)) => {
                self.eat_assert(Token::Symbol(RAngle))?;
                Some((Binop::Gt, Box::new(self.term()?)))
            }
            Some(Token::Symbol(Equal)) => {
                self.eat_assert(Token::Symbol(Equal))?;
                Some((Binop::Equal, Box::new(self.term()?)))
            }
            _ => None,
        };
        Ok(Expr::new(term, cdr))
    }

    fn term(&mut self) -> Result<Term, String> {
        if let Some(Token::Ident(_)) = self.tokens.peek() {
            let name = self.eat_ident()?;
            match self.tokens.peek() {
                Some(Token::Symbol(LBracket)) => {
                    self.eat_assert(Token::Symbol(LBracket))?;
                    let expr = self.expr()?;
                    self.eat_assert(Token::Symbol(RBracket))?;
                    return Ok(Term::WithIdx(name, Box::new(expr)));
                }
                Some(Token::Symbol(Dot)) | Some(Token::Symbol(LParen)) => {
                    let subroutine_call = self.subroutine_call(name)?;
                    return Ok(Term::SubRoutineCall(subroutine_call));
                }
                _ => {
                    return Ok(Term::ValName(name));
                }
            }
        }
        match self.tokens.next() {
            Some(Token::IntegerConstant(num)) => Ok(Term::IntegerConstant(num)),
            Some(Token::StringConstant(s)) => Ok(Term::StringConstant(s.clone())),
            Some(Token::Keyword(True)) => Ok(Term::KeywordConstant(KeywordConstant::True)),
            Some(Token::Keyword(False)) => Ok(Term::KeywordConstant(KeywordConstant::False)),
            Some(Token::Keyword(Null)) => Ok(Term::KeywordConstant(KeywordConstant::Null)),
            Some(Token::Keyword(This)) => Ok(Term::KeywordConstant(KeywordConstant::This)),
            Some(Token::Symbol(LParen)) => {
                let expr = self.expr()?;
                self.eat_assert(Token::Symbol(RParen))?;
                Ok(Term::Expr(Box::new(expr)))
            }
            Some(Token::Symbol(Minus)) => {
                let term = self.term()?;
                Ok(Term::WithUnary(Unop::Minus, Box::new(term)))
            }
            Some(Token::Symbol(Tilde)) => {
                let term = self.term()?;
                Ok(Term::WithUnary(Unop::BitNot, Box::new(term)))
            }
            _ => Err("Error".to_string()),
        }
    }

    fn subroutine_call(&mut self, name: Rc<String>) -> Result<SubRoutineCall, String> {
        let (obj_name, routine_name) = {
            if let Some(Token::Symbol(Dot)) = self.tokens.peek() {
                self.eat_assert(Token::Symbol(Dot))?;
                let routine_name = self.eat_ident()?;
                (Some(name), routine_name)
            } else {
                (None, name)
            }
        };
        self.eat_assert(Token::Symbol(LParen))?;
        let mut args = Vec::new();
        loop {
            match self.tokens.peek() {
                Some(Token::Symbol(RParen)) => {
                    self.eat_assert(Token::Symbol(RParen))?;
                    break;
                }
                _ => {
                    args.push(self.expr()?);
                    if let Some(Token::Symbol(Comma)) = self.tokens.peek() {
                        self.eat_assert(Token::Symbol(Comma))?;
                    }
                }
            }
        }
        Ok(SubRoutineCall::new(obj_name, routine_name, args))
    }

    fn eat_assert(&mut self, token: Token) -> Result<Token, String> {
        match self.tokens.peek() {
            Some(t) => {
                if token == *t {
                    Ok(self.tokens.next().unwrap())
                } else {
                    Err("Error".to_string())
                }
            }
            _ => Err("Error".to_string()),
        }
    }

    fn eat_type(&mut self, allow_void: bool) -> Result<Type, String> {
        let ty = match self.tokens.peek() {
            Some(Token::Keyword(Int)) => Type::Int,
            Some(Token::Keyword(Char)) => Type::Char,
            Some(Token::Keyword(Boolean)) => Type::Boolean,
            Some(Token::Ident(s)) => Type::Class(s.clone()),
            Some(Token::Keyword(Void)) => {
                if allow_void {
                    Type::Void
                } else {
                    return Err("Error".to_string());
                }
            }
            _ => return Err("Error".to_string()),
        };
        self.tokens.next();
        Ok(ty)
    }

    fn eat_ident(&mut self) -> Result<Rc<String>, String> {
        let name = match self.tokens.peek() {
            Some(Token::Ident(s)) => s.clone(),
            _ => return Err("Error".to_string()),
        };
        self.tokens.next();
        Ok(name)
    }
}
