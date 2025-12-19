use std::{collections::HashMap, iter::zip, mem::take};

use crate::{
    ast::{Expr, Extern, FuncDecl, Program},
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
        panic!("NameError: undefined variable '{}' in current scope.", name);
    }

    pub fn from_var_type(&self, var_type: &VarType) -> IRType {
        match var_type {
            VarType::Number => IRType::Number,
            VarType::Bool => IRType::Bool,
            VarType::Str => IRType::String,
            VarType::Void => IRType::Void,
            _ => unimplemented!(),
        }
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
            panic!(
                "NameError: variable '{}' already declared in this scope.",
                name
            );
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
        for expr in &program.body {
            match expr {
                Expr::FuncDecl(decl) => {
                    self.func_decl(decl.clone());
                }
                Expr::Extern(ext) => {
                    self.extern_decl(ext.clone());
                }
                _ => {}
            }
        }

        for expr in program.body {
            match expr {
                Expr::FuncDecl(decl) => {
                    self.compile_fn(decl);
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
                ctx.declare_var(decl.name.clone(), ctx.from_var_type(&decl.typ));
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
                if typ != ctx.get_operand_type(&right) {
                    panic!(
                        "TypeError: unsupported operation for '{:?}' and '{:?}'",
                        typ,
                        ctx.get_operand_type(&right)
                    )
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
                let res_tmp = ctx.new_tmp(ctx.get_operand_type(&argument));
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
                }
                ctx.new_tmp(IRType::Void)
            }
            Expr::If(i) => {
                let label_else = ctx.new_label("else");
                let label_end = ctx.new_label("endif");

                let cond = self.compile_expr(*i.condition, ctx);

                let then_op = self.compile_expr(*i.then_branch.clone(), ctx);

                let res_tmp = ctx.new_tmp(ctx.get_operand_type(&then_op));

                ctx.instructions.push(Instruction {
                    op: Op::JumpIfFalse,
                    dst: None,
                    src1: Some(cond),
                    src2: Some(Operand::Label(label_else.clone())),
                });

                if !matches!(*i.then_branch, Expr::Stmt(_)) {
                    ctx.enter_scope();
                }

                ctx.instructions.push(Instruction {
                    op: Op::Move,
                    dst: Some(res_tmp.clone()),
                    src1: Some(then_op),
                    src2: None,
                });

                if !matches!(*i.then_branch, Expr::Stmt(_)) {
                    ctx.exit_scope();
                }

                ctx.instructions.push(Instruction {
                    op: Op::Jump,
                    dst: None,
                    src1: Some(Operand::Label(label_end.clone())),
                    src2: None,
                });

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

                    let else_op = self.compile_expr(*else_expr.clone(), ctx);

                    let then_type = ctx.get_operand_type(&res_tmp);
                    let else_type = ctx.get_operand_type(&else_op);

                    if then_type != else_type {
                        panic!(
                            "TypeError: If-Else expression branches must have the same return type: {:?} vs {:?}",
                            then_type, else_type
                        );
                    }

                    ctx.instructions.push(Instruction {
                        op: Op::Move,
                        dst: Some(res_tmp.clone()),
                        src1: Some(else_op),
                        src2: None,
                    });

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

                res_tmp
            }
            Expr::While(w) => {
                let label_start = ctx.new_label("while_start");
                let label_end = ctx.new_label("while_end");
                let cond = self.compile_expr(*w.condition, ctx);
                ctx.instructions.push(Instruction {
                    op: Op::JumpIfFalse,
                    dst: None,
                    src1: Some(cond),
                    src2: Some(Operand::Label(label_end.clone())),
                });
                ctx.instructions.push(Instruction {
                    op: Op::Label(label_start.clone()),
                    dst: None,
                    src1: None,
                    src2: None,
                });
                if !matches!(*w.body, Expr::Stmt(_)) {
                    ctx.enter_scope();
                }
                let while_body = self.compile_expr(*w.body.clone(), ctx);
                if !matches!(*w.body, Expr::Stmt(_)) {
                    ctx.exit_scope();
                }
                ctx.instructions.push(Instruction {
                    op: Op::Jump,
                    dst: None,
                    src1: Some(Operand::Label(label_start)),
                    src2: None,
                });
                ctx.instructions.push(Instruction {
                    op: Op::Label(label_end.clone()),
                    dst: None,
                    src1: None,
                    src2: None,
                });
                ctx.new_tmp(IRType::Void)
            }
            Expr::For(f) => {
                unimplemented!()
            }
            Expr::FuncDecl(_) => {
                panic!("SyntaxError: cannot declare a function in a function");
            }
            Expr::FuncCall(call) => {
                let func = self.find_func(&call.name);
                if call.args.len() != func.params.len() {
                    panic!(
                        "TypeError: expected {} arguments, got {}",
                        call.args.len(),
                        func.params.len()
                    );
                }
                let res_tmp = ctx.new_tmp(ctx.from_var_type(&call.ret_type));
                let mut n = 0;
                for (arg, param) in zip(call.args.iter(), func.params.iter()) {
                    let operand = self.compile_expr(arg.clone(), ctx);
                    if ctx.get_operand_type(&operand) != param.1 {
                        panic!(
                            "TypeError: unexpected type {:?}, expected {:?}",
                            ctx.get_operand_type(&operand),
                            param.1
                        );
                    }
                    ctx.instructions.push(Instruction {
                        op: Op::Arg(n),
                        dst: None,
                        src1: Some(operand),
                        src2: None,
                    });
                    n += 1;
                }
                ctx.instructions.push(Instruction {
                    op: Op::Call,
                    dst: Some(res_tmp.clone()),
                    src1: Some(Operand::Function(call.name)),
                    src2: None,
                });
                res_tmp
            }
            Expr::ArrayAccess(aa) => {
                unimplemented!();
            }
            Expr::ArrayAssign(aa) => {
                unimplemented!();
            }
            Expr::Extern(ext) => {
                panic!("SyntaxError: cannot extern a function in a function");
            }
            Expr::Goto(goto) => {
                ctx.instructions.push(Instruction {
                    op: Op::Jump,
                    dst: None,
                    src1: Some(Operand::Label(goto.label)),
                    src2: None,
                });
                ctx.new_tmp(IRType::Void)
            }
            Expr::Label(label) => {
                ctx.instructions.push(Instruction {
                    op: Op::Label(label.name),
                    dst: None,
                    src1: None,
                    src2: None,
                });
                ctx.new_tmp(IRType::Void)
            }
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

    fn func_decl(&mut self, decl: FuncDecl) {
        let mut temp_ctx = Context::new();
        let params: Vec<(Operand, IRType)> = decl
            .params
            .iter()
            .enumerate()
            .map(|(i, (name, typ))| (Operand::Var(name.clone()), temp_ctx.from_var_type(typ)))
            .collect();

        let ret_type = temp_ctx.from_var_type(&decl.ret_type);

        self.functions.push(IRFunction {
            name: decl.name.clone(),
            params,
            ret_type,
            instructions: Vec::new(),
            is_pub: decl.is_pub,
            is_external: false,
        });
    }

    fn compile_fn(&mut self, decl: FuncDecl) {
        let name = decl.name.clone();
        let mut ctx = Context::new();
        ctx.enter_scope();

        let func = self.find_func(&name).clone();
        let params = func.params.clone();

        for (op, typ) in &params {
            if let Operand::Var(name) = op {
                ctx.declare_var(name.clone(), typ.clone());
            }
        }

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

        if let Some(func) = self.functions.iter_mut().find(|f| f.name == name) {
            func.instructions = take(&mut ctx.instructions);
        }
    }

    fn extern_decl(&mut self, ext: Extern) -> () {
        let name = ext.name;
        let params: Vec<(Operand, IRType)> = ext
            .params
            .into_iter()
            .enumerate()
            .map(|(i, typ)| {
                let temp_ctx = Context::new();
                let param_name = format!("a{}", i);
                (Operand::Var(param_name), temp_ctx.from_var_type(&typ))
            })
            .collect();

        let ret_type = Context::new().from_var_type(&ext.ret_type);

        let signature = IRFunction {
            name: name.clone(),
            params,
            ret_type,
            instructions: Vec::new(),
            is_pub: false,
            is_external: true,
        };
        self.functions.push(signature);
    }

    fn find_func(&self, name: &String) -> IRFunction {
        for func in self.functions.iter().rev() {
            if func.name == *name {
                return func.to_owned();
            }
        }
        panic!("NameError: undefined function '{}' in current scope", name);
    }
}
