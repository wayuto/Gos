import { Compiler, GVM, Lexer, Parser } from "@wayuto/gos";
import { assertEquals } from "@std/assert/equals";

Deno.test("Compiler", () => {
  const code = `
    let x = (1 + 2) * 3
    out x
  `;

  const lexer = new Lexer(code);
  const parser = new Parser(lexer);
  const ast = parser.parse();
  const compiler = new Compiler();
  const { chunk, maxSlot } = compiler.compile(ast);
  console.log(chunk, maxSlot);
  assertEquals(chunk, {
    code: new Uint8Array([0, 0, 0, 1, 3, 0, 2, 5, 2, 0, 1, 0, 12, 13]),
    constants: [1, 2, 3],
  });
});
