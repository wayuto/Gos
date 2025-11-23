import { Compiler, Lexer, Parser, Preprocessor } from "@wayuto/gos";
import type { Chunk } from "./compiler.ts";
import type { Literal } from "../token.ts";

export const compile = async (
  sourceFile: string,
  outputFile?: string,
): Promise<void> => {
  if (!outputFile) {
    const lastDotIndex = sourceFile.lastIndexOf(".");
    outputFile = lastDotIndex !== -1
      ? sourceFile.substring(0, lastDotIndex) + ".gbc"
      : sourceFile + ".gbc";
  }

  const src = await Deno.readTextFile(sourceFile);
  const preprocessor = new Preprocessor(src);
  const code = await preprocessor.preprocess();
  const lexer = new Lexer(code);
  const parser = new Parser(lexer);
  const ast = parser.parse();
  const compiler = new Compiler();
  const { chunk, maxSlot } = compiler.compile(ast);

  const binaryData = serializeToBinary(chunk, maxSlot);

  await Deno.writeFile(outputFile, binaryData);

  console.log(
    `Compiled ${sourceFile} to ${outputFile} (${binaryData.length} bytes)`,
  );
};

const serializeToBinary = (chunk: Chunk, maxSlot: number): Uint8Array => {
  const encoder = new TextEncoder();

  let totalSize = 0;
  totalSize += 4;
  totalSize += 2;
  totalSize += 4;
  totalSize += chunk.code.length;
  totalSize += 2;

  let constantsSize = 0;
  for (const constant of chunk.constants) {
    constantsSize += 1;
    if (typeof constant === "number") {
      constantsSize += 8;
    } else if (typeof constant === "boolean") {
      constantsSize += 1;
    } else if (typeof constant === "string") {
      const encoded = encoder.encode(constant as string);
      constantsSize += 2 + encoded.length;
    }
  }

  totalSize += constantsSize;

  totalSize += 2;

  const buffer = new ArrayBuffer(totalSize);
  const dataView = new DataView(buffer);
  let offset = 0;

  dataView.setUint8(offset++, 0x47);
  dataView.setUint8(offset++, 0x4F);
  dataView.setUint8(offset++, 0x53);
  dataView.setUint8(offset++, 0x42);

  dataView.setUint16(offset, 1, true);
  offset += 2;

  dataView.setUint32(offset, chunk.code.length, true);
  offset += 4;

  for (let i = 0; i < chunk.code.length; i++) {
    dataView.setUint8(offset++, chunk.code[i]);
  }

  dataView.setUint16(offset, chunk.constants.length, true);
  offset += 2;

  for (const constant of chunk.constants) {
    if (constant === undefined || constant === null) {
      dataView.setUint8(offset++, 0);
    } else if (typeof constant === "number") {
      dataView.setUint8(offset++, 1);
      dataView.setFloat64(offset, constant as number, true);
      offset += 8;
    } else if (typeof constant === "boolean") {
      dataView.setUint8(offset++, 2);
      dataView.setUint8(offset++, (constant as boolean) ? 1 : 0);
    } else if (typeof constant === "string") {
      dataView.setUint8(offset++, 3);
      const encoded = encoder.encode(constant as string);
      dataView.setUint16(offset, encoded.length, true);
      offset += 2;
      for (let i = 0; i < encoded.length; i++) {
        dataView.setUint8(offset++, encoded[i]);
      }
    }
  }

  dataView.setUint16(offset, maxSlot, true);

  return new Uint8Array(buffer);
};

export const load = async (
  file: string,
): Promise<{ chunk: Chunk; maxSlot: number }> => {
  const data = await Deno.readFile(file);
  const dataView = new DataView(data.buffer);
  const decoder = new TextDecoder();
  let offset = 0;

  const magic = [
    dataView.getUint8(offset++),
    dataView.getUint8(offset++),
    dataView.getUint8(offset++),
    dataView.getUint8(offset++),
  ];

  if (
    magic[0] !== 0x47 || magic[1] !== 0x4F || magic[2] !== 0x53 ||
    magic[3] !== 0x42
  ) {
    throw new Error("Invalid bytecode file format");
  }

  const version = dataView.getUint16(offset, true);
  offset += 2;

  if (version !== 1) {
    throw new Error(`Unsupported bytecode version: ${version}`);
  }

  const codeLength = dataView.getUint32(offset, true);
  offset += 4;

  const code = new Uint8Array(codeLength);
  for (let i = 0; i < codeLength; i++) {
    code[i] = dataView.getUint8(offset++);
  }

  const constantsCount = dataView.getUint16(offset, true);
  offset += 2;

  const constants: Literal[] = [];
  for (let i = 0; i < constantsCount; i++) {
    const type = dataView.getUint8(offset++);

    switch (type) {
      case 0:
        constants.push(undefined);
        break;
      case 1:
        constants.push(dataView.getFloat64(offset, true));
        offset += 8;
        break;
      case 2:
        constants.push(dataView.getUint8(offset++) !== 0);
        break;
      case 3: {
        const strLength = dataView.getUint16(offset, true);
        offset += 2;
        const strBytes = new Uint8Array(
          data.buffer,
          data.byteOffset + offset,
          strLength,
        );
        constants.push(decoder.decode(strBytes));
        offset += strLength;
        break;
      }
      default:
        throw new Error(`Unknown constant type: ${type}`);
    }
  }

  const maxSlot = dataView.getUint16(offset, true);

  return {
    chunk: {
      code,
      constants,
    },
    maxSlot,
  };
};
