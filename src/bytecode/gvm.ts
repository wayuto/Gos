import type { Chunk } from "./compiler.ts";
import { Op } from "./bytecode.ts";
import type { Literal } from "../lexer.ts";
import { err } from "../utils.ts";

/**GosVM */
export class GVM {
  private ip = 0;
  private stack: Literal[] = [];
  private slots: Literal[] = [];

  constructor(private chunk: Chunk, private maxSlot: number) {
    this.slots = new Array<Literal>(this.maxSlot);
  }

  public run = () => {
    while (true) {
      const op = this.chunk.code[this.ip++];
      switch (op) {
        case Op.LOAD_CONST: {
          const idx = this.chunk.code[this.ip++];
          this.stack.push(this.chunk.constants[idx]);
          break;
        }
        case Op.LOAD_VAR: {
          const slot = this.chunk.code[this.ip++];
          this.stack.push(this.slots[slot]);
          break;
        }
        case Op.STORE_VAR: {
          const slot = this.chunk.code[this.ip++];
          this.slots[slot] = this.stack.pop();
          break;
        }
        case Op.ADD: {
          const left = this.stack.pop();
          const right = this.stack.pop();
          const val = (left as number) + (right as number);
          this.stack.push(val);
          break;
        }
        case Op.SUB: {
          const left = this.stack.pop();
          const right = this.stack.pop();
          const val = (left as number) - (right as number);
          this.stack.push(val);
          break;
        }
        case Op.MUL: {
          const left = this.stack.pop();
          const right = this.stack.pop();
          const val = (left as number) * (right as number);
          this.stack.push(val);
          break;
        }
        case Op.DIV: {
          const left = this.stack.pop();
          const right = this.stack.pop();
          const val = (left as number) / (right as number);
          this.stack.push(val);
          break;
        }
        case Op.OUT: {
          const value = this.stack.pop();
          console.log(value);
          break;
        }
        case Op.NEG: {
          const val = this.stack.pop();
          this.stack.push(-(val as number));
          break;
        }
        case Op.POS: {
          break;
        }
        case Op.INC: {
          const val = this.stack.pop();
          this.stack.push((val as number) + 1);
          break;
        }
        case Op.DEC: {
          const val = this.stack.pop();
          this.stack.push((val as number)--);
          break;
        }
        case Op.LOG_NOT: {
          const val = this.stack.pop();
          this.stack.push(!val);
          break;
        }
        case Op.HALT: {
          return;
        }
        default: {
          return err(
            "GVM",
            `Unknown operator: ${op}`,
          );
        }
      }
    }
  };
}
