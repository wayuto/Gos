import { Preprocessor } from "@wayuto/gos";
import { assertEquals } from "@std/assert/equals";

Deno.test("Preprocessor", async () => {
  const src = '$import "examples/fibonacci.gos"';

  const preprocessor = new Preprocessor(src);
  const code = await preprocessor.preprocess();
  const expected = await Deno.readTextFile("examples/fibonacci.gos");
  assertEquals(code, expected);
});
