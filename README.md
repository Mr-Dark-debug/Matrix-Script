# MatrixScript

MatrixScript is a JIT-compiled language for high-performance linear algebra.

## Prerequisites

- Rust (2021 Edition)
- LLVM 17 (libllvm17 and llvm-17-dev)

## Build and Run

To build the project:
```bash
cargo build --release
```

To run a script:
```bash
cargo run -- examples/math.ms
```

## Example

```rust
fn main() {
    let a = 10.0;
    let b = 20.0;
    return a * b + 5.0;
}
```
