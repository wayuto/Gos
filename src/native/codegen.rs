use std::{collections::HashMap, mem::take};

use crate::native::{IRConst, IRFunction, IRProgram, Instruction, Op, Operand};

macro_rules! assemble {
        ($buf:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
            $buf.push_str(&format!(concat!("\n", $fmt) $(, $arg)*))
        };
}

pub struct CodeGen {
    program: IRProgram,
    text: String,
    data: String,
    vars: HashMap<String, usize>,
    str_cnt: usize,
    stack_ptr: usize,
    arg_reg: Vec<String>,
}

impl CodeGen {
    pub fn new(program: IRProgram) -> Self {
        Self {
            program,
            text: String::new(),
            data: String::new(),
            vars: HashMap::new(),
            str_cnt: 0,
            stack_ptr: 0,
            arg_reg: vec![
                "rdi".to_string(),
                "rsi".to_string(),
                "rdx".to_string(),
                "rcx".to_string(),
                "r8".to_string(),
                "r9".to_string(),
            ],
        }
    }

    pub fn compile(&mut self) -> String {
        assemble!(self.text, "section .text");
        assemble!(self.data, "section .data");
        for func in take(&mut self.program.functions) {
            self.compile_fn(func);
        }
        take(&mut self.data) + &self.text
    }

    fn compile_code(&mut self, code: Instruction) -> () {
        match code.op {
            Op::Move => {
                self.load(&code.src1.unwrap(), "rax");
                let dst = code.dst.as_ref().unwrap();
                assemble!(self.text, "mov [rbp - {}], rax", self.get_offset(dst));
            }
            Op::Load => {
                self.load(&code.src1.unwrap(), "rax");
                let offset = self.get_offset(code.dst.as_ref().unwrap());
                assemble!(self.text, "mov [rbp - {}], rax", offset);
            }
            Op::Store => {
                let offset = self.get_offset(code.dst.as_ref().unwrap());
                self.load(&code.src1.unwrap(), "rax");
                assemble!(self.text, "mov [rbp - {}], rax", offset);
            }
            Op::Add | Op::Sub | Op::Mul | Op::Div => {
                let dst = code.dst.as_ref().unwrap();
                let src1 = code.src1.as_ref().unwrap();
                let src2 = code.src2.as_ref().unwrap();

                self.load(src1, "rax");
                self.load(src2, "rbx");
                match code.op {
                    Op::Add => assemble!(self.text, "add rax, rbx"),
                    Op::Sub => assemble!(self.text, "sub rax, rbx"),
                    Op::Mul => assemble!(self.text, "imul rax, rbx"),
                    Op::Div => {
                        assemble!(self.text, "cqo");
                        assemble!(self.text, "idiv rbx")
                    }
                    _ => panic!(),
                }
                assemble!(self.text, "mov [rbp - {}], rax", self.get_offset(dst));
            }
            Op::Eq | Op::Ne | Op::Gt | Op::Ge | Op::Lt | Op::Le => {
                let dst = code.dst.as_ref().unwrap();
                let src1 = code.src1.as_ref().unwrap();
                let src2 = code.src2.as_ref().unwrap();

                self.load(src1, "rax");
                self.load(src2, "rbx");

                assemble!(self.text, "cmp rax, rbx");
                match code.op {
                    Op::Eq => assemble!(self.text, "sete al"),
                    Op::Ne => assemble!(self.text, "setne al"),
                    Op::Gt => assemble!(self.text, "setg al"),
                    Op::Ge => assemble!(self.text, "setge al"),
                    Op::Gt => assemble!(self.text, "setl al"),
                    Op::Ge => assemble!(self.text, "setle al"),
                    _ => {}
                }
                assemble!(self.text, "movzx rax, al");
                assemble!(self.text, "mov [rbp - {}], rax", self.get_offset(dst));
            }
            Op::Arg(n) => {
                let op = code.src1.as_ref().unwrap();
                let offset = self.get_offset(op);

                if n < 6 {
                    assemble!(self.text, "mov {}, [rbp - {}]", self.arg_reg[0], offset);
                    return;
                }
                assemble!(self.text, "mov rax, [rbp - {}]", offset);
                assemble!(self.text, "push rax")
            }
            Op::Call => {
                let dst = code.dst.as_ref().unwrap();
                let func = code.src1.as_ref().unwrap();

                let offset = self.get_offset(dst);

                match func {
                    Operand::Function(name) => {
                        assemble!(self.text, "call {}", name);
                        assemble!(self.text, "mov [rbp - {}], rax", offset)
                    }
                    _ => panic!("NameError: '{:?}' is not a function", func),
                }
            }
            Op::Label(lbl) => {
                assemble!(self.text, "{}:", lbl);
            }
            Op::Jump => {
                let lbl = code.src1.as_ref().unwrap();
                if let Operand::Label(lbl) = lbl {
                    assemble!(self.text, "jmp {}", lbl)
                }
            }
            Op::JumpIfFalse => {
                let src1 = code.src1.as_ref().unwrap();
                let src2 = code.src2.as_ref().unwrap();
                let offset = self.get_offset(src1);
                let lbl = match src2 {
                    Operand::Label(s) => s,
                    _ => panic!("TypeError: '{:?}' is not a label", src2),
                };
                assemble!(self.text, "mov rax, [rbp - {}]", offset);
                assemble!(self.text, "cmp rax, 0");
                assemble!(self.text, "je {}", lbl);
            }
            Op::Return => {
                let val = code.src1.as_ref().unwrap();
                self.load(val, "rax");
                assemble!(self.text, "leave");
                assemble!(self.text, "ret");
            }
            _ => panic!("UnknowError: unknown TAC: {:?}", code),
        }
    }

    fn compile_fn(&mut self, func: IRFunction) -> () {
        if func.is_external {
            assemble!(self.text, "extern {}", func.name);
            return;
        }

        self.vars.clear();
        let mut offset = 0;

        for (param, _typ) in &func.params {
            if let Operand::Var(name) = param {
                if !self.vars.contains_key(name) {
                    offset += 8;
                    self.vars.insert(name.clone(), offset);
                }
            }
        }

        for inst in &func.instructions {
            let ops = [&inst.dst, &inst.src1, &inst.src2];
            for op_opt in ops {
                if let Some(Operand::Temp(id, _)) = op_opt {
                    let temp_name = format!("_tmp_{}", id);
                    if !self.vars.contains_key(&temp_name) {
                        offset += 8;
                        self.vars.insert(temp_name, offset);
                    }
                } else if let Some(Operand::Var(name)) = op_opt {
                    if !self.vars.contains_key(name) {
                        offset += 8;
                        self.vars.insert(name.clone(), offset);
                    }
                }
            }
        }
        let stack_size = (offset + 15) & !15;

        if func.is_pub {
            assemble!(self.text, "global {}", func.name);
        }
        assemble!(self.text, "{}:", func.name);
        assemble!(self.text, "push rbp");
        assemble!(self.text, "mov rbp, rsp");
        if stack_size > 0 {
            assemble!(self.text, "sub rsp, {}", stack_size);
        }
        for (i, (param, _)) in func.params.iter().enumerate() {
            if i < 6 {
                let offset = self.get_offset(param);
                assemble!(self.text, "mov [rbp - {}], {}", offset, self.arg_reg[i]);
            }
        }
        for code in func.instructions {
            self.compile_code(code);
        }
    }

    fn load(&mut self, op: &Operand, reg: &str) -> () {
        match op {
            Operand::ConstIdx(idx) => {
                let val = &self.program.constants[*idx];
                match val.to_owned() {
                    IRConst::Number(n) => {
                        assemble!(self.text, "mov {}, {}", reg, n);
                    }
                    IRConst::Bool(b) => {
                        assemble!(self.text, "mov {}, {}", reg, b);
                    }
                    IRConst::Void => {
                        assemble!(self.text, "mov {}, {}", reg, 0);
                    }
                    IRConst::Str(s) => {
                        let s_lbl = self.alloc_str(s);
                        assemble!(self.text, "mov {}, {}", reg, s_lbl);
                    }
                    _ => unimplemented!(),
                }
            }
            Operand::Var(_) | Operand::Temp(_, _) => {
                let offset = self.get_offset(op);
                assemble!(self.text, "mov {}, [rbp - {}]", reg, offset);
            }
            _ => unimplemented!(),
        }
    }

    fn alloc_str(&mut self, s: String) -> String {
        let s_lbl = format!(".S.{}", self.str_cnt);
        self.str_cnt += 1;
        assemble!(self.data, "{} db {}, 0", s_lbl, s);
        s_lbl
    }

    fn get_offset(&self, op: &Operand) -> usize {
        match op {
            Operand::Var(name) => *self
                .vars
                .get(name)
                .unwrap_or_else(|| panic!("NameError: undefined variable: {}", name)),
            Operand::Temp(id, _) => {
                let temp_key = format!("_tmp_{}", id);
                *self
                    .vars
                    .get(&temp_key)
                    .unwrap_or_else(|| panic!("NameError: undefined temporary: T{}", id))
            }
            _ => panic!("UnknownError: unknown operand: {:?}", op),
        }
    }
}
