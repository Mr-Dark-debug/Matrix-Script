# Project: MatrixScript (The Tensor-Native Language)

## 1. Core Mission
We are building a high-performance, Domain Specific Language (DSL) called "MatrixScript". 
**The Goal:** Outperform Python in matrix multiplication tasks by compiling directly to native machine code using LLVM and SIMD optimizations.
**The Vibe:** Minimalist, math-heavy, and blazingly fast.

## 2. Technical Stack (Strict)
* **Host Language:** Rust (2021 Edition).
* **Backend:** LLVM (via `inkwell` crate).
* **Build System:** Cargo.
* **Version Control:** Git.

## 3. Architecture Rules
* **Lexer:** Hand-written or using `logos`. No heavy regex libraries if possible.
* **Parser:** Recursive Descent. We need clear error messages, so avoid parser generators like ANTLR.
* **Codegen:** Direct translation to LLVM IR.
    * **Matrices:** Must be represented as a struct `{ double* data, i64 rows, i64 cols }`.
    * **Operations:** Matrix math must use loops or vector intrinsics (SIMD), not external C library calls.
* **JIT:** Use LLVM's ORC JIT for immediate code execution.

## 4. Quality Standards
* **Documentation:** Every public struct, enum, and function MUST have `///` Rustdoc comments explaining its purpose.
* **Testing:** Every feature (parsing, codegen, math) must have a corresponding integration test in the `tests/` folder.
* **Error Handling:** Use `anyhow` or `thiserror`. Never use `unwrap()` in parser or compiler logic; return proper Results.
