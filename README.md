# **The Alum Programming Language**

Alum is a lightweight, efficient programming language which is implemented in
**Rust**

## **🚀 Installation**

### **Prerequisites**

Alum is designed for **x86_64 Linux** environments. Ensure you have the
following installed:

- **Rust & Cargo** (2024 edition)
- **NASM** (The Netwide Assembler)
- **ld** (GNU Linker)

### **Setup**

Clone the repository and run the automated installation script to set up the
`al` binary and the standard library:

```bash
# Clone the repository  
git clone --depth 1 https://github.com/wayuto/Alum ~/Alum
cd ~/Alum

# Run the installation script  
# This installs the 'al' CLI, builds the standard library,   
# and moves 'libal.a' to /usr/local/lib  
sh ./install.sh
```

## **🛠 Features & Syntax**

### **Basic Types**

- `int`: 64-bit signed integer.
- `flt`: 64-bit floating-point number (IEEE 754 double precision).
- `str`: String type.
- `bool`: Boolean logic (true / false).
- `arr<N>`: Fixed-size arrays (e.g., arr<5>).
- `void`: Used for functions that do not return a value.

### **Variables**

```
let x: int = 42 
let pi: flt = 3.14159
let message: str = "Hello, Alum!"
let flags: arr<3> = [1, 2, 3] 
let inferred: arr<_> = [1, 2, 3, 4] # Length inferred as 4
let filled: arr<5> = [0] # [0, 0, 0, 0, 0]
```

Floating-point numbers support all standard arithmetic operations (+, -, *, /)
and comparisons (==, !=, >, >=, <, <=).

### **Control Flow**

Alum supports modern control flow structures, including if-else expressions and
loops.

```
# If-Else as an expression
let result: str = if x > 10 "High" else "Low"

# While loop
while x > 0 { x-- }

# Range-based For loop ($import "array" before using `n..m`)
for i in 0..10 { println(itoa(i)) }
```

### **Block Scopes**

In Alum, code blocks are expressions. The last value in a block is returned as
the block's value.

```
let computed: int = { 
  let a: int = 10 
  let b: int = 20 
  a + b # This is the block's value
}
```

## **🔧 Preprocessor Directives**

Alum includes a preprocessor that supports directives for code organization and
reuse.

```alum
$ifndef MACRO
$define MACRO 1

$define DEBUG 1
$ifdef DEBUG
println("Debug mode enabled.")
$endif

$endif
```

Macros are simple text replacements that occur during preprocessing. They can be
used for constants, simple expressions, or code snippets. Parameterized macros
are not currently supported.

## **📚 Standard Library (alum-std)**

The Alum Standard Library provides essential functionality out of the box. Use
`$import` to include them.

| Module      | Key Functions                                                           |
| :---------- | :---------------------------------------------------------------------- |
| **io**      | print, println, input, read, write, fopen, fclose, lseek, fread, fwrite |
| **math**    | abs, sqrt, max, min, pow, fact, PI, E                                   |
| **string**  | strlen, strcpy, strcat, memcpy, memset                                  |
| **convert** | itoa, atoi, ftoa, atof                                                  |
| **array**   | range, find                                                             |
| **stdlib**  | syscall, exit                                                           |

## **💻 Language Examples**

### **Hello World**

Save this as `hello.al`:

```
$import "io"

pub fun main(): int {
  println("Hello world!") 
  return 0
}
```

Run with:

```bash
al hello.al
./hello
```

### **Recursive Fibonacci**

```
$import "io"
$import "convert"

fun fib(n: int): int {
  if n < 2 return n
  return fib(n - 1) + fib(n - 2)
}

pub fun main(): int {
  let n: int = fib(40)
  println(
    itoa(n)
  ) 
  return 0 
}
```

## **🔗 FFI & Interoperability**

Alum is designed to play well with C. You can declare external functions and
call them directly. **Calling a C function in Alum:**

# Declare the external C function

```
extern println(str): int

pub fun main(): int {
  println("Hello world!")
  return 0
}
```

**Exposing a Alum function to C:**

# Use `pub` to make it visible to the linker

```
pub fun add(x: int, y: int): int {
  return x + y
}
```

## **📊 Benchmark**

### Environment

- CPU: Intel i5-8265U (8 cores @ 3.900GHz)
- Memory: 8GB DDR4 (7647MiB)
- Architecture: x86_64
- Operating System: Arch Linux
- Kernel Version: 6.18.1-zen1-2-zen

### Test Content

Performance test comparing three programming languages using tail recursion to
compute the 1000th Fibonacci number:

- Alum Native 0.5.2: Alum language compiled to native
- executable C (GCC -O3): C language compiled with GCC's highest optimization
  level
- Python 3.13.11: Python interpreted execution

```
➜  fibonacci1000 ./run.sh # in tty
Benchmark 1: ./fib1000
  Time (mean ± σ):     260.3 µs ±  67.6 µs    [User: 165.4 µs, System: 30.3 µs]
  Range (min … max):   157.3 µs … 498.0 µs    12679 runs
 
Benchmark 2: ./a.out
  Time (mean ± σ):     604.2 µs ± 186.1 µs    [User: 290.9 µs, System: 208.3 µs]
  Range (min … max):   337.4 µs … 1204.2 µs    4869 runs
 
Benchmark 3: python fib1000.py
  Time (mean ± σ):      12.9 ms ±   0.4 ms    [User: 10.1 ms, System: 2.6 ms]
  Range (min … max):    12.4 ms …  14.6 ms    237 runs
 
Summary
  ./fib1000 ran
    2.32 ± 0.94 times faster than ./a.out
   49.55 ± 12.95 times faster than python fib1000.py
```

## **⚙️ CLI Reference**

```bash
The Alum programming language compiler

Usage: al [OPTIONS] <input_files>...

Arguments:
  <input_files>...  Input source files

Options:
  -o, --output <file>  Place output in <file>
  -E                   Preprocess only; do not compile, assemble or link
  -S                   Compile only; do not assemble or link
  -c                   Compile and assemble, but do not link
      --dump-ast       Dump AST representation
      --dump-ir        Dump IR representation
      --nostdlib       Do not link with standard library
  -v, --verbose        Verbose output
  -h, --help           Print help
  -V, --version        Print version
```
