# MatrixScript

MatrixScript is a high-performance, JIT-compiled Domain Specific Language (DSL) designed for linear algebra and tensor operations. Built with **Rust** and **LLVM**, it aims to provide a "blazingly fast" alternative for matrix math by compiling directly to native machine code with SIMD optimizations.

## 🚀 Project Overview

The core philosophy of MatrixScript is to treat Matrices as first-class citizens. Unlike general-purpose languages where matrices are libraries, here they are native types with optimized LLVM IR generation.

**Key Features:**
- **JIT Compilation**: Uses LLVM's ORC JIT for immediate execution of scripts.
- **Native Matrix Types**: Matrices are allocated on the heap and operations are performed via generated machine code loops.
- **Recursive Descent Parser**: A hand-written, easy-to-debug parser.
- **No External Runtime Dependencies**: Matrix operations are generated inline (Phase 2) without calling heavy C libraries (like BLAS... yet).

---

## 📂 Folder Structure

The project is organized as follows:

```
matrix_script/
├── src/
│   ├── compiler/          # Core compiler logic
│   │   ├── ast.rs         # Abstract Syntax Tree definitions (Expr, Stmt, Function)
│   │   ├── lexer.rs       # Token definitions using `logos`
│   │   ├── parser.rs      # Recursive Descent Parser implementation
│   │   ├── codegen.rs     # LLVM IR Code Generator (the heavy lifter)
│   │   ├── jit.rs         # JIT Execution Engine wrapper
│   │   └── mod.rs         # Module exports
│   └── main.rs            # CLI entry point (not shown in file list but implied)
├── examples/              # Example MatrixScript source files (.ms)
│   ├── math.ms            # Basic scalar math example
│   └── matrix_test.ms     # Matrix addition example
├── tests/                 # Integration tests
│   └── test_matrix.rs     # Verifies JIT compilation and execution
├── local_libs/            # Local library dependencies (if any)
├── Cargo.toml             # Rust project configuration
└── AGENTS.md              # Instructions for AI agents working on this repo
```

---

## 🛠️ Architecture & Internals

### 1. Lexer (`lexer.rs`)
Uses the `logos` crate to tokenize the input source.
- **Tokens**: `Let`, `Return`, `Fn`, identifiers, numbers, operators (`+`, `-`, `*`, `/`), and structural symbols (`[`, `]`, `{`, `}`, `,`).
- Skips whitespace automatically.

### 2. Parser (`parser.rs`)
A handwritten recursive descent parser that converts a stream of Tokens into an Abstract Syntax Tree (AST).
- **Structure**: Parses `Program` -> `Vec<Function>` -> `Vec<Stmt>`.
- **Expressions**: Handles precedence for binary operators. Supports scalar numbers and matrix literals.
- **Matrix Parsing**: Supports both nested lists `[[1, 2], [3, 4]]` and vector-style `[1, 2, 3]`.

### 3. AST (`ast.rs`)
Defines the data structures representing the code.
- `Expr::MatrixLiteral(Vec<Vec<Expr>>)`: The representation of a matrix in the tree.
- `Stmt::Let`: Variable bindings.
- `Function`: Named function definitions.
- Implements `fmt::Display` for easy debugging and formatted output.

### 4. CodeGen (`codegen.rs`)
The heart of the compiler. It translates the AST into LLVM Intermediate Representation (IR).
- **Matrix Layout**:
  ```c
  struct Matrix {
      double* data;  // Pointer to flat heap array
      i64 rows;
      i64 cols;
  }
  ```
- **Functions**:
  - `compile_matrix_literal`: Allocates memory on the heap (using `build_array_malloc`), populates it with values, and returns a pointer to the `Matrix` struct.
  - `compile_matrix_add`: Generates a raw LLVM IR loop to perform element-wise addition. It detects if operands are matrices (via pointer type checking) or scalars (via float type checking).
  - **Type Inference**: A basic pass scans the function body to determine if the return type should be `f64` (Scalar) or `Matrix*` (Pointer), adjusting the LLVM function signature accordingly.

### 5. JIT (`jit.rs`)
Wraps `inkwell`'s ExecutionEngine.
- Compiles the LLVM Module to native machine code in memory.
- Executes the `main` function.

---

## 📖 Language Reference

### Variables
Defined using `let`. Variables are immutable by default (in current scope).
```rust
let x = 10.0;
let M = [[1.0, 2.0], [3.0, 4.0]];
```

### Matrices
Native support for 2D matrices.
```rust
let A = [[1.0, 0.0], [0.0, 1.0]]; // 2x2 Identity
let B = [1.0, 2.0, 3.0];          // 1x3 Row Vector
```

### Functions
Currently supports a `main` function.
```rust
fn main() {
    let A = [[1.0, 2.0], [3.0, 4.0]];
    let B = [[5.0, 6.0], [7.0, 8.0]];
    return A + B;
}
```

---

## 🔮 Upcoming & Planned Features

The development is divided into phases. We are currently completing **Phase 2**.

- [x] **Phase 1 (MVP)**: Basic scalar arithmetic, parser, simple JIT.
- [x] **Phase 2 (Matrices)**: Matrix types, heap allocation, matrix addition, CLI support.
- [ ] **Phase 3 (Advanced Ops)**:
    - Matrix Multiplication (Dot Product).
    - Transposition.
    - Matrix Slicing/Indexing (e.g., `A[0, 1]`).
- [ ] **Phase 4 (Memory Management)**:
    - Garbage Collection (currently we leak memory).
    - Stack allocation optimization for small matrices.
- [ ] **Phase 5 (Language Features)**:
    - Function arguments.
    - Control flow (`if`, `while`).
    - Standard Library (print, math functions).

---

## ⚙️ Building and Running

### Prerequisites
- **Rust** (2021 Edition)
- **LLVM 17**: You must have LLVM 17 installed and `llvm-config` available in your path.
  - *Note: The project is configured to use `llvm-sys` 170. If you have a different version, you may need to adjust `Cargo.toml`.*

### Build
```bash
cargo build --release
```

### Run Example
```bash
cargo run -- examples/matrix_test.ms
```

### Run Tests
```bash
cargo test
```
