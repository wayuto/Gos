import type { Expression, Program } from "../ast.ts";
import { type Literal, TokenType } from "../lexer.ts";
import { err } from "../utils.ts";
import { Op } from "./bytecode.ts";

export interface Chunk {
  code: Uint8Array;
  constants: Literal[];
}

/**Compiler */
export class Compiler {
  private constants: Literal[] = [];
  private codes: number[] = [];
  private locals = new Map<string, number>();
  private nextSlot = 0;

  private emit = (op: Op, ...arg: number[]): void => {
    this.codes.push(op);
    if (arg !== undefined) this.codes.push(...arg);
  };

  public compile = (program: Program): { chunk: Chunk; maxSlot: number } => {
    for (const expr of program.body) {
      this.compileExpr(expr);
    }

    this.emit(Op.HALT);

    return {
      chunk: { code: new Uint8Array(this.codes), constants: this.constants },
      maxSlot: this.nextSlot,
    };
  };

  private compileExpr = (
    expr: Expression,
  ): void => {
    switch (expr.type) {
      case "Val": {
        const val = expr.value;
        this.constants.push(val);
        this.emit(Op.LOAD_CONST, this.constants.length - 1);
        break;
      }
      case "Var": {
        const slot = this.locals.get(expr.name);
        if (slot === undefined) {
          return err(
            "Compiler",
            `Variable '${expr.name}' has not been defined`,
          );
        }
        this.emit(Op.LOAD_VAR, slot);
        break;
      }
      case "VarDecl": {
        this.compileExpr(expr.value);
        const slot = this.nextSlot++;
        this.locals.set(expr.name, slot);
        this.emit(Op.STORE_VAR, slot);
        break;
      }
      case "BinOp": {
        this.compileExpr(expr.left);
        this.compileExpr(expr.right);
        switch (expr.op) {
          case TokenType.OP_ADD: {
            this.emit(Op.ADD);
            break;
          }
          case TokenType.OP_SUB: {
            this.emit(Op.SUB);
            break;
          }
          case TokenType.OP_MUL:
            this.emit(Op.MUL);
            break;
          case TokenType.OP_DIV: {
            this.emit(Op.DIV);
            break;
          }
        }
        break;
      }
      case "UnaryOp": {
        this.compileExpr(expr.argument);
        switch (expr.op) {
          case TokenType.LOG_NOT: {
            this.emit(Op.LOG_NOT);
            break;
          }
          case TokenType.OP_INC: {
            this.emit(Op.INC);
            break;
          }
          case TokenType.OP_DEC: {
            this.emit(Op.DEC);
            break;
          }
          default: {
            expr.op === TokenType.OP_NEG
              ? this.emit(Op.NEG)
              : this.emit(Op.POS);
          }
        }
        break;
      }
      case "Out": {
        this.compileExpr(expr.value);
        this.emit(Op.OUT);
        break;
      }
      default: {
        return err("Compiler", `Unknown node type: ${expr.type}`);
      }
    }
  };
}
