mod parser;

use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use parser::{Op::*, Segment::*, *};

pub struct AsmGenerator {
    asm: Vec<Cow<'static, str>>,
    label_count: u16,
}

impl AsmGenerator {
    pub fn new(no_init: bool) -> Self {
        let mut generator = Self {
            asm: Vec::new(),
            label_count: 0,
        };
        if no_init {
            return generator;
        }
        generator.asm.push(Cow::Borrowed("@256"));
        generator.asm.push(Cow::Borrowed("D=A"));
        generator.asm.push(Cow::Borrowed("@SP"));
        generator.asm.push(Cow::Borrowed("M=D"));
        generator.call("Sys.init", 0);
        generator
    }

    pub fn gen(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref().to_str().ok_or("Invalid file path")?;

        let file = File::open(path).map_err(|_| format! {"Can't open file: {}", path})?;
        let commands = parse_vm(BufReader::new(file))?;
        for com in commands {
            match com {
                Command::Arithmetic(op) => match op {
                    Add | Sub | Eq_ | Gt | Lt | And | Or => self.binop(op),
                    Not | Neg => self.uniop(op),
                },
                Command::Push(seg, offset) => self.push(seg, offset, path),
                Command::Pop(seg, offset) => self.pop(seg, offset, path),
                Command::Label(label) => self.label(&label),
                Command::Goto(label) => self.goto(&label),
                Command::IfGoto(label) => self.if_goto(&label),
                Command::Function(label, n_locs) => self.function(&label, n_locs),
                Command::Call(label, arity) => self.call(&label, arity),
                Command::Return => self.return_(),
            }
        }
        Ok(())
    }

    pub fn flush(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let f = File::create(path.as_ref())
            .map_err(|_| format! {"Invalid file path: {:?}", path.as_ref()})?;
        let mut writer = BufWriter::new(f);
        for line in &self.asm {
            writer
                .write_all(line.as_bytes())
                .map_err(|_| "Can't write file")?;
            writer
                .write_all("\n".as_bytes())
                .map_err(|_| "Can't write file")?;
        }
        Ok(())
    }

    fn binop(&mut self, op: Op) {
        self.pop_dreg();
        self.asm.push(Cow::Borrowed("@SP"));
        self.asm.push(Cow::Borrowed("M=M-1"));
        self.asm.push(Cow::Borrowed("A=M"));
        match op {
            Add => self.asm.push(Cow::Borrowed("D=M+D")),
            Sub => self.asm.push(Cow::Borrowed("D=M-D")),
            Eq_ | Gt | Lt => {
                let true_label = self.label_count();
                let end_label = self.label_count();
                self.asm.push(Cow::Borrowed("D=M-D"));
                self.asm
                    .push(Cow::Owned(format! {"@COMPTRUE_{}", true_label}));
                match op {
                    Eq_ => self.asm.push(Cow::Borrowed("D;JEQ")),
                    Gt => self.asm.push(Cow::Borrowed("D;JGT")),
                    Lt => self.asm.push(Cow::Borrowed("D;JLT")),
                    _ => unreachable!(),
                }
                self.asm.push(Cow::Borrowed("D=0"));
                self.asm
                    .push(Cow::Owned(format! {"@COMPEND_{}", end_label}));
                self.asm.push(Cow::Borrowed("0;JMP"));
                self.asm
                    .push(Cow::Owned(format! {"(COMPTRUE_{})", true_label}));
                self.asm.push(Cow::Borrowed("D=-1"));
                self.asm
                    .push(Cow::Owned(format! {"(COMPEND_{})", end_label}));
            }
            And => self.asm.push(Cow::Borrowed("D=M&D")),
            Or => self.asm.push(Cow::Borrowed("D=M|D")),
            _ => unreachable!(),
        };
        self.push_dreg()
    }

    fn uniop(&mut self, op: Op) {
        self.pop_dreg();
        match op {
            Not => self.asm.push(Cow::Borrowed("D=!D")),
            Neg => self.asm.push(Cow::Borrowed("D=-D")),
            _ => unimplemented!(),
        }
        self.push_dreg();
    }

    fn push_const(&mut self, val: u16) {
        self.asm.push(Cow::Owned(format! {"@{}", val}));
        self.asm.push(Cow::Borrowed("D=A"));
        self.push_dreg();
    }

    fn push(&mut self, segment: Segment, offset: u16, path: impl AsRef<Path>) {
        let reg = match segment {
            Argument => "@ARG",
            Local => "@LCL",
            This => "@THIS",
            That => "@THAT",
            Temp => "@R5",
            Pointer => "@THIS",
            Static => return self.push_static(offset, path),
            Constant => return self.push_const(offset),
        };
        self.asm.push(Cow::Owned(format! {"@{}", offset}));
        self.asm.push(Cow::Borrowed("D=A"));
        self.asm.push(Cow::Borrowed(reg));
        match segment {
            Argument | Local | This | That => self.asm.push(Cow::Borrowed("A=M+D")),
            Pointer | Temp => self.asm.push(Cow::Borrowed("A=A+D")),
            Static | Constant => unreachable!(),
        }
        self.asm.push(Cow::Borrowed("D=M"));
        self.push_dreg();
    }

    fn pop(&mut self, segment: Segment, offset: u16, path: impl AsRef<Path>) {
        let reg = match segment {
            Argument => "@ARG",
            Local => "@LCL",
            This => "@THIS",
            That => "@THAT",
            Temp => "@R5",
            Pointer => "@THIS",
            Static => return self.pop_static(offset, path),
            Constant => unreachable!(),
        };
        self.asm.push(Cow::Owned(format! {"@{}", offset}));
        self.asm.push(Cow::Borrowed("D=A"));
        self.asm.push(Cow::Borrowed(reg));
        match segment {
            Argument | Local | This | That => self.asm.push(Cow::Borrowed("D=M+D")),
            Pointer | Temp => self.asm.push(Cow::Borrowed("D=A+D")),
            Static | Constant => unreachable!(),
        }
        self.asm.push(Cow::Borrowed("@R14"));
        self.asm.push(Cow::Borrowed("M=D"));
        self.pop_dreg();
        self.asm.push(Cow::Borrowed("@R14"));
        self.asm.push(Cow::Borrowed("A=M"));
        self.asm.push(Cow::Borrowed("M=D"));
    }

    fn label(&mut self, label: &str) {
        self.asm.push(Cow::Owned(format! {"({})", label}));
    }

    fn goto(&mut self, label: &str) {
        self.asm.push(Cow::Owned(format! {"@{}", label}));
        self.asm.push(Cow::Borrowed("0;JMP"));
    }

    fn if_goto(&mut self, label: &str) {
        self.pop_dreg();
        self.asm.push(Cow::Owned(format! {"@{}", label}));
        self.asm.push(Cow::Borrowed("D;JNE"));
    }

    fn function(&mut self, label: &str, n_locs: u16) {
        self.label(label);
        self.asm.push(Cow::Borrowed("D=0"));
        (0..n_locs).for_each(|_| self.push_dreg());
    }

    fn call(&mut self, label: &str, arity: u16) {
        let return_label = format! {"FUNC_RETURN_{}", self.label_count()};
        self.asm.push(Cow::Owned(format! {"@{}", return_label}));
        self.asm.push(Cow::Borrowed("D=A"));
        self.push_dreg();

        for s in ["@LCL", "@ARG", "@THIS", "@THAT"].iter() {
            self.asm.push(Cow::Borrowed(s));
            self.asm.push(Cow::Borrowed("D=M"));
            self.push_dreg();
        }
        self.asm.push(Cow::Borrowed("@SP"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Owned(format! {"@{}", arity + 5}));
        self.asm.push(Cow::Borrowed("D=D-A"));
        self.asm.push(Cow::Borrowed("@ARG"));
        self.asm.push(Cow::Borrowed("M=D"));
        self.asm.push(Cow::Borrowed("@SP"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@LCL"));
        self.asm.push(Cow::Borrowed("M=D"));
        self.asm.push(Cow::Owned(format! {"@{}", label}));
        self.asm.push(Cow::Borrowed("0;JMP"));
        self.label(&return_label);
    }

    fn return_(&mut self) {
        self.asm.push(Cow::Borrowed("@LCL"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@5"));
        self.asm.push(Cow::Borrowed("A=D-A"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@R13"));
        self.asm.push(Cow::Borrowed("M=D"));

        self.pop_dreg();
        self.asm.push(Cow::Borrowed("@ARG"));
        self.asm.push(Cow::Borrowed("A=M"));
        self.asm.push(Cow::Borrowed("M=D"));

        self.asm.push(Cow::Borrowed("@ARG"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@SP"));
        self.asm.push(Cow::Borrowed("M=D+1"));

        self.asm.push(Cow::Borrowed("@LCL"));
        self.asm.push(Cow::Borrowed("A=M-1"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@THAT"));
        self.asm.push(Cow::Borrowed("M=D"));

        self.asm.push(Cow::Borrowed("@LCL"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@2"));
        self.asm.push(Cow::Borrowed("A=D-A"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@THIS"));
        self.asm.push(Cow::Borrowed("M=D"));

        self.asm.push(Cow::Borrowed("@LCL"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@3"));
        self.asm.push(Cow::Borrowed("A=D-A"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@ARG"));
        self.asm.push(Cow::Borrowed("M=D"));

        self.asm.push(Cow::Borrowed("@LCL"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@4"));
        self.asm.push(Cow::Borrowed("A=D-A"));
        self.asm.push(Cow::Borrowed("D=M"));
        self.asm.push(Cow::Borrowed("@LCL"));
        self.asm.push(Cow::Borrowed("M=D"));

        self.asm.push(Cow::Borrowed("@R13"));
        self.asm.push(Cow::Borrowed("A=M"));
        self.asm.push(Cow::Borrowed("0;JMP"));
    }

    fn push_static(&mut self, val: u16, path: impl AsRef<Path>) {
        self.asm.push(Cow::Owned(
            format! {"@{}.{}", path.as_ref().file_stem().unwrap().to_str().unwrap(), val},
        ));
        self.asm.push(Cow::Borrowed("D=M"));
        self.push_dreg();
    }

    fn push_dreg(&mut self) {
        self.asm.push(Cow::Borrowed("@SP"));
        self.asm.push(Cow::Borrowed("A=M"));
        self.asm.push(Cow::Borrowed("M=D"));
        self.asm.push(Cow::Borrowed("@SP"));
        self.asm.push(Cow::Borrowed("M=M+1"));
    }

    fn pop_static(&mut self, val: u16, path: impl AsRef<Path>) {
        self.pop_dreg();
        self.asm.push(Cow::Owned(
            format! {"@{}.{}", path.as_ref().file_stem().unwrap().to_str().unwrap(), val},
        ));
        self.asm.push(Cow::Borrowed("M=D"));
    }

    fn pop_dreg(&mut self) {
        self.asm.push(Cow::Borrowed("@SP"));
        self.asm.push(Cow::Borrowed("M=M-1"));
        self.asm.push(Cow::Borrowed("A=M"));
        self.asm.push(Cow::Borrowed("D=M"));
    }

    fn label_count(&mut self) -> u16 {
        let count = self.label_count;
        self.label_count += 1;
        count
    }
}
