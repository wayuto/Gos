use std::{collections::HashMap, mem::take, process::exit};

use crate::{
    ast::{Expr, FuncDecl, Program},
    native::{IRConst, IRFunction, IRProgram, IRType, Instruction, Op, Operand},
    token::{Literal, TokenType, VarType},
};

#[derive(Debug, Clone)]
struct Symbol {
    pub name: String,
    pub ir_type: IRType,
}

type Scope = HashMap<String, Symbol>;

struct Context {
    pub instructions: Vec<Instruction>,
    pub tmp_cnt: usize,
    pub scope: Vec<Scope>,
    pub label_cnt: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            tmp_cnt: 0,
            scope: Vec::new(),
            label_cnt: 0,
        }
    }

    pub fn new_tmp(&mut self, tmp_type: IRType) -> Operand {
        self.tmp_cnt += 1;
        Operand::Temp(self.tmp_cnt - 1, tmp_type)
    }

    pub fn new_label(&mut self, name: &str) -> String {
        self.label_cnt += 1;
        format!("{}_{:X}", name, self.label_cnt - 1)
    }

    pub fn enter_scope(&mut self) {
        self.scope.push(Scope::new());
    }

    pub fn exit_scope(&mut self) {
        self.scope.pop().expect("Tried to pop the root scope.");
    }

    fn get_var_type(&self, name: &str) -> IRType {
        for scope in self.scope.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return symbol.ir_type.clone();
            }
        }
        panic!("Error: Undefined variable '{}' in current scope.", name);
    }

    pub fn get_operand_type(&self, operand: &Operand) -> IRType {
        match operand {
            Operand::Const(c) => match c {
                IRConst::Number(_) => IRType::Number,
                IRConst::Bool(_) => IRType::Bool,
                IRConst::Str(_) => IRType::String,
                IRConst::Void => IRType::Void,
            },
            Operand::Var(name) => self.get_var_type(&name),
            Operand::Temp(_, t) => t.to_owned(),
            Operand::Label(_) => IRType::Void,
            Operand::Function(_) => IRType::Void,
            Operand::ConstIdx(_) => IRType::Void,
        }
    }

    pub fn declare_var(&mut self, name: String, ir_type: IRType) {
        let current_scope = self.scope.last_mut().unwrap();
        if current_scope.contains_key(&name) {
            panic!("Error: Variable '{}' already declared in this scope.", name);
        }
        current_scope.insert(name.clone(), Symbol { name, ir_type });
    }
}

pub struct IRGen {
    functions: Vec<IRFunction>,
    constants: Vec<IRConst>,
    constant_pool: HashMap<IRConst, usize>,
}

impl IRGen {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            constants: Vec::new(),
            constant_pool: HashMap::new(),
        }
    }

    pub fn compile(&mut self, program: Program) -> IRProgram {
        for expr in program.body {
            match expr {
                Expr::FuncDecl(decl) => {
                    self.func_decl(decl);
                }
                Expr::Val(val) => {
                    self.global_constant(val.value);
                }
                _ => {}
            }
        }
        IRProgram {
            functions: take(&mut self.functions),
            constants: take(&mut self.constants),
        }
    }

    fn get_const_index(&mut self, constant: IRConst) -> usize {
        if let Some(&index) = self.constant_pool.get(&constant) {
            return index;
        }

        let index = self.constants.len();
        self.constants.push(constant.clone());
        self.constant_pool.insert(constant, index);
        index
    }

    fn compile_expr(&mut self, expr: Expr, ctx: &mut Context) -> Operand {
        match expr {
            Expr::Val(val) => {
                let (ir_const, ir_type) = match val.value {
                    Literal::Number(n) => (IRConst::Number(n), IRType::Number),
                    Literal::Bool(b) => (IRConst::Number(if b { 1 } else { 0 }), IRType::Number),
                    Literal::Str(s) => (IRConst::Str(s), IRType::String),
                    Literal::Void => return ctx.new_tmp(IRType::Void),
                    Literal::Array(_, _) => unimplemented!(),
                };

                let res_tmp = ctx.new_tmp(ir_type);

                let const_idx = self.get_const_index(ir_const);

                ctx.instructions.push(Instruction {
                    op: Op::Move,
                    dst: Some(res_tmp.clone()),
                    src1: Some(Operand::ConstIdx(const_idx)),
                    src2: None,
                });
                res_tmp
            }
            Expr::VarDecl(decl) => {
                let value = self.compile_expr(*decl.value, ctx);
                ctx.declare_var(
                    decl.name.clone(),
                    match decl.typ {
                        VarType::Number => IRType::Number,
                        VarType::Bool => IRType::Bool,
                        VarType::Str => IRType::String,
                        VarType::Void => IRType::Void,
                        _ => unimplemented!(),
                    },
                );
                ctx.instructions.push(Instruction {
                    op: Op::Store,
                    dst: Some(Operand::Var(decl.name)),
                    src1: Some(value),
                    src2: None,
                });
                ctx.new_tmp(IRType::Void)
            }
            Expr::VarMod(modi) => {
                let value = self.compile_expr(*modi.value, ctx);
                let typ = ctx.get_operand_type(&value);
                if typ != ctx.get_var_type(&modi.name) {
                    panic!("TypeError: unexpected type: {:?}", typ);
                }
                ctx.instructions.push(Instruction {
                    op: Op::Store,
                    dst: Some(Operand::Var(modi.name)),
                    src1: Some(value),
                    src2: None,
                });
                ctx.new_tmp(IRType::Void)
            }
            Expr::Var(var) => {
                let var_type = ctx.get_var_type(&var.name);
                let res_tmp = ctx.new_tmp(var_type);
                ctx.instructions.push(Instruction {
                    op: Op::Load,
                    dst: Some(res_tmp.clone()),
                    src1: Some(Operand::Var(var.name)),
                    src2: None,
                });
                res_tmp
            }
            Expr::BinOp(bin) => {
                let left = self.compile_expr(*bin.left, ctx);
                let right = self.compile_expr(*bin.right, ctx);
                let typ = ctx.get_operand_type(&left);
                if typ != ctx.get_operand_type(&left) {
                    panic!("")
                }
                let res_tmp = ctx.new_tmp(typ);
                ctx.instructions.push(Instruction {
                    op: match bin.operator {
                        TokenType::ADD => Op::Add,
                        TokenType::SUB => Op::Sub,
                        TokenType::MUL => Op::Mul,
                        TokenType::DIV => Op::Div,
                        TokenType::COMPEQ => Op::Eq,
                        TokenType::COMPNE => Op::Ne,
                        TokenType::COMPGT => Op::Gt,
                        TokenType::COMPGE => Op::Ge,
                        TokenType::COMPLT => Op::Lt,
                        TokenType::COMPLE => Op::Le,
                        TokenType::COMPAND => Op::And,
                        TokenType::COMPOR => Op::Or,
                        TokenType::LOGAND => Op::LAnd,
                        TokenType::LOGOR => Op::LOr,
                        TokenType::LOGXOR => Op::Xor,
                        _ => panic!(),
                    },
                    dst: Some(res_tmp.clone()),
                    src1: Some(left),
                    src2: Some(right),
                });
                res_tmp
            }
            Expr::UnaryOp(unary) => {
                let argument = self.compile_expr(*unary.argument, ctx);
                let res_tmp = ctx.new_tmp(IRType::Number);
                ctx.instructions.push(Instruction {
                    op: match unary.operator {
                        TokenType::NEG => Op::Neg,
                        TokenType::INC => Op::Inc,
                        TokenType::DEC => Op::Dec,
                        TokenType::SIZEOF => Op::SizeOf,
                        _ => panic!(),
                    },
                    dst: Some(res_tmp.clone()),
                    src1: Some(argument),
                    src2: None,
                });
                res_tmp
            }
            Expr::Stmt(stmt) => {
                ctx.enter_scope();

                let body_len = stmt.body.len();

                for i in 0..body_len.saturating_sub(1) {
                    self.compile_expr(stmt.body[i].clone(), ctx);
                }

                let result_operand = if let Some(last_expr) = stmt.body.last() {
                    self.compile_expr(last_expr.clone(), ctx)
                } else {
                    ctx.new_tmp(IRType::Void)
                };
                ctx.exit_scope();
                result_operand
            }
            Expr::Return(ret) => {
                if let Some(value) = ret.value {
                    let value_op = self.compile_expr(*value, ctx);

                    ctx.instructions.push(Instruction {
                        op: Op::Return,
                        dst: None,
                        src1: Some(value_op),
                        src2: None,
                    });
                    ctx.new_tmp(IRType::Void)
                } else {
                    ctx.new_tmp(IRType::Void)
                }
            }
            Expr::If(i) => {
                let label_else = ctx.new_label("else");
                let label_end = ctx.new_label("endif");
                let cond = self.compile_expr(*i.condition, ctx);
                ctx.instructions.push(Instruction {
                    op: Op::JumpIfFalse,
                    dst: None,
                    src1: Some(cond),
                    src2: Some(Operand::Label(label_else.clone())),
                });
                if !matches!(*i.then_branch, Expr::Stmt(_)) {
                    ctx.enter_scope();
                }
                let then_branch = self.compile_expr(*i.then_branch.clone(), ctx);
                if !matches!(*i.then_branch, Expr::Stmt(_)) {
                    ctx.exit_scope();
                }
                if i.else_branch.is_some() {
                    ctx.instructions.push(Instruction {
                        op: Op::Jump,
                        dst: None,
                        src1: Some(Operand::Label(label_end.clone())),
                        src2: None,
                    });
                }

                ctx.instructions.push(Instruction {
                    op: Op::Label(label_else),
                    dst: None,
                    src1: None,
                    src2: None,
                });
                if let Some(else_expr) = i.else_branch {
                    if !matches!(*else_expr, Expr::Stmt(_)) {
                        ctx.enter_scope();
                    }
                    self.compile_expr(*else_expr.clone(), ctx);
                    if !matches!(*else_expr, Expr::Stmt(_)) {
                        ctx.exit_scope();
                    }
                }
                ctx.instructions.push(Instruction {
                    op: Op::Label(label_end),
                    dst: None,
                    src1: None,
                    src2: None,
                });
                let res_tmp = ctx.new_tmp(ctx.get_operand_type(&then_branch));
                res_tmp
            }
            _ => unimplemented!("{:?}", expr),
        }
    }

    fn global_constant(&mut self, literal: Literal) {
        match literal {
            Literal::Number(n) => self.constants.push(IRConst::Number(n)),
            Literal::Bool(b) => self.constants.push(IRConst::Bool(b)),
            Literal::Str(s) => self.constants.push(IRConst::Str(s)),
            _ => panic!("Invalid global constant type."),
        }
    }

    fn func_decl(&mut self, decl: FuncDecl) -> () {
        let name = decl.name.clone();
        let mut ctx = Context::new();
        ctx.enter_scope();
        let params: Vec<(Operand, IRType)> = decl
            .params
            .into_iter()
            .map(|(param, typ)| {
                (
                    Operand::Var(param),
                    match typ {
                        VarType::Number => IRType::Number,
                        VarType::Bool => IRType::Bool,
                        VarType::Str => IRType::String,
                        VarType::Void => IRType::Void,
                        _ => unimplemented!(),
                    },
                )
            })
            .collect();

        let body = *decl.body;
        let last_op = self.compile_expr(body, &mut ctx);
        ctx.exit_scope();

        if ctx.instructions.last().map(|i| i.op.clone()) != Some(Op::Return) {
            ctx.instructions.push(Instruction {
                op: Op::Return,
                dst: None,
                src1: Some(last_op),
                src2: None,
            });
        }

        self.functions.push(IRFunction {
            name: name,
            params,
            ret_type: IRType::Number,
            instructions: ctx.instructions,
            is_pub: decl.is_pub,
        });
    }
}
