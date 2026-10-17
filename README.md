# RnR Compiler

[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![Target](https://img.shields.io/badge/Architecture-MIPS32-blue.svg)](https://en.wikipedia.org/wiki/MIPS_architecture)
[![License](https://img.shields.io/badge/License-Academic-lightgrey.svg)](LICENSE)

An end-to-end compiler, static type checker, AST interpreter, and MIPS assembly code generator for **RnR** (a strongly-typed Rust-like language), implemented in Rust.

Developed as the capstone compiler project for the **Compilers and Formal Languages** course (D7050E) at [Luleå University of Technology (LTU)](https://www.ltu.se/).

---

## Architecture Overview

The RnR compiler is split into a modular pipeline featuring both an interpreted tree-walking runtime and an optimizing stack-frame code generator targeting MIPS assembly:

```
                      +-------------------+
                      |   Source (.rnr)   |
                      +---------+---------+
                                |
                                v
                      +-------------------+
                      |  Lexer & Parser   | (syn / AST tokens)
                      +---------+---------+
                                |
                                v
                      +-------------------+
                      |  AST Definition   | (ast.rs, ebnf.md)
                      +---------+---------+
                                |
                                v
                      +-------------------+
                      | Static TypeCheck  | (Semantic Analysis)
                      +----+---------+----+
                           |         |
          +----------------+         +----------------+
          |                                           |
          v (Interpreted path)                        v (Compiled path)
+-------------------+                       +-------------------+
|  AST Interpreter  |                       |  MIPS Code Gen    |
|   (vm.rs / Stack) |                       | (code_generator)  |
+-------------------+                       +---------+---------+
                                                      |
                                                      v
                                            +-------------------+
                                            |   MIPS Assembly   |
                                            +---------+---------+
                                                      |
                                                      v
                                            +-------------------+
                                            |  MIPS Emulator VM |
                                            +-------------------+
```

### Compiler Phases

1. **Parser & AST (`src/parse.rs`, `src/ast.rs`)**:
   - Parses RnR source code into structured AST nodes.
   - Enforces operator precedence and associativity across binary arithmetic, logical, and comparison operations.
   - Handles statements, blocks, function declarations, parameters, and expressions.
2. **Static Type Checker (`src/type_check.rs`)**:
   - Validates semantic correctness ahead of execution.
   - Tracks types using an environment (`TypeEnv`) across scopes.
   - Verifies variable types, return types, branch type consistency in `if/else`, and operand constraints.
3. **AST Interpreter / VM (`src/vm.rs`)**:
   - Tree-walking evaluation engine with a scope stack for local bindings and shadowing.
   - Supports first-class function environments and function hoisting.
4. **MIPS Code Generator (`src/code_generator.rs`)**:
   - Emits MIPS32 assembly instructions.
   - Implements stack-machine execution for expression evaluation.
   - Manages call frames (prologue/epilogue, `$fp`, `$sp`, `$ra`), local variable allocation, and function parameters.
   - Supports nested functions with unique label generation to prevent name collisions.
5. **Execution Runtime (`mips` crate)**:
   - Executes generated MIPS instructions in an emulated virtual machine, extracting the final program result from register `$v0`.

---

## Language Features

RnR implements a clean, expressive subset of Rust:

- **Primitive Types**: `i32`, `bool`, `()`, and `String` (parsing & VM).
- **Variables & Mutability**: `let` and `let mut` variable bindings with lexical scoping and shadowing.
- **Control Flow**:
  - `if` / `else if` / `else` conditional expressions.
  - `while` loops.
- **Functions & Recursion**:
  - Typed parameters and return values.
  - Recursive functions (e.g., Fibonacci, Euclidean algorithms).
  - Nested function definitions with local lexical scoping.
  - Function hoisting (functions callable before declaration within a block).
- **Expressions & Operators**:
  - Arithmetic: `+`, `-`, `*`, `/`
  - Logic: `&&`, `||`, `!`
  - Comparison: `==`, `<`, `>`
  - Unary negation: `-`, `!`

For the complete formal grammar definition, see [`docs/ebnf.md`](docs/ebnf.md).

---

## Example Program

Recursive Fibonacci calculation in RnR ([`examples/fibonacci.rnr`](examples/fibonacci.rnr)):

```rust
fn fib(n: i32) -> i32 {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() -> i32 {
    fib(6) // Evaluates to 8
}
```

Nested functions and scoping ([`examples/nested_func.rnr`](examples/nested_func.rnr)):

```rust
fn main() -> i32 {
    let mut x: i32 = add(3, 5);
    x
}

fn add(a: i32, b: i32) -> i32 {
    fn add(a: i32) -> i32 {
        a
    }
    a + add(b)
}
```

---

## Getting Started

### Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (Edition 2021)

### Installation

```bash
git clone https://github.com/Rohanito12/rnr-compiler.git
cd rnr-compiler
cargo build --release
```

The compiled binary will be located at `target/release/rnr`.

---

## CLI Usage

The compiler features an interactive command-line interface built with `clap`:

```
rnr [OPTIONS]
```

### Options

| Flag | Alias | Description |
| :--- | :---- | :---------- |
| `-i, --input <FILE>` | | Input `.rnr` source file (defaults to `main.rnr`) |
| `-t, --type_check` | | Runs static type checking on the program |
| `-v, --interpret` | `--vm`, `--virtual_machine` | Executes program using the AST tree-walking interpreter |
| `-c, --code_gen` | `--code_gen` | Runs backend MIPS assembly code generation |
| `-r, --run` | | Compiles to MIPS and runs on the MIPS virtual machine |
| `-m, --asm <FILE>` | `--asm` | Dumps generated MIPS assembly to specified file |
| `-a, --ast <FILE>` | | Dumps parsed AST representation to specified file |
| `-h, --help` | | Displays help information |

### Examples

**Execute program on the MIPS Virtual Machine:**
```bash
cargo run -- -i examples/fibonacci.rnr -r
```

**Interpret with the AST VM:**
```bash
cargo run -- -i examples/fibonacci.rnr -v
```

**Run Static Type Checking:**
```bash
cargo run -- -i examples/test_string.rnr -t
```

**Generate MIPS Assembly and save to file:**
```bash
cargo run -- -i examples/nested_func.rnr -c -m nested.asm
```

**Export AST representation:**
```bash
cargo run -- -i examples/fibonacci.rnr -a fib.ast
```

---

## Testing

The project includes an extensive test suite with over 900 lines of integration tests covering parsing, precedence, static type analysis, AST interpretation, and MIPS code generation:

```bash
cargo test
```

---

## Project Structure

```
.
├── Cargo.toml                  # Project metadata & dependencies
├── docs/
│   ├── ebnf.md                 # Formal EBNF grammar specification
│   ├── course_instructions.md  # Original university lab specifications
│   ├── reflection.md           # Engineering reflections across compiler labs
│   └── changelog.md            # Development changelog
├── examples/                   # Sample RnR programs & compiler output dumps
│   ├── fibonacci.rnr
│   ├── nested_func.rnr
│   ├── test_while.rnr
│   ├── test_string.rnr
│   ├── test_type_error.rnr
│   ├── output.asm              # Sample generated MIPS assembly
│   └── output.ast              # Sample AST dump
├── src/
│   ├── ast.rs                  # AST data structures (Types, Exprs, Stmts)
│   ├── ast_traits.rs           # Formatting & display traits
│   ├── code_generator.rs       # MIPS32 code generator & frame manager
│   ├── common.rs               # Evaluation traits
│   ├── error.rs                # Error handling definitions
│   ├── intrinsics.rs           # Built-in functions (e.g. println!)
│   ├── lib.rs                  # Module exports
│   ├── main.rs                 # CLI entry point
│   ├── parse.rs                # Token stream parser into AST
│   ├── test_util.rs            # Testing macros & helpers
│   ├── type_check.rs           # Static semantic analysis & type environment
│   └── vm.rs                   # AST tree-walking interpreter
└── tests/
    └── integration_tests.rs    # Comprehensive test suite (960+ lines)
```

---

## Academic Context

This project was built for course **D7050E: Compiler Construction and Formal Languages** at **Luleå University of Technology**.
