import { Context } from "../src/ast-walking/context.ts";
import { Interpreter } from "../src/ast-walking/interpreter.ts";
import { Lexer } from "../src/lexer.ts";
import { Parser } from "../src/parser.ts";

const src = `

`;

const lexer = new Lexer(src);
const parser = new Parser(lexer);
const ast = parser.parse();

const context = new Context();
const interpreter = new Interpreter(context);
interpreter.execute(ast);
