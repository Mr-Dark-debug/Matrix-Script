use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use inkwell::context::Context as InkwellContext;
use matrix_script::compiler; // Use the library module
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "MatrixScript")]
#[command(version = "1.0")]
#[command(about = "A JIT-compiled language for high-performance linear algebra", long_about = None)]
struct Cli {
    /// The file to run
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let source = fs::read_to_string(&cli.file)
        .with_context(|| format!("Failed to read file {:?}", cli.file))?;

    // 1. Lexing & Parsing
    let mut parser = compiler::parser::Parser::new(&source)?;
    let program = parser.parse_program()?;

    // 2. LLVM Codegen
    let context = InkwellContext::create();
    let mut codegen = compiler::codegen::CodeGen::new(&context, "matrix_script_module");
    codegen.compile_program(&program)?;

    // 3. JIT Execution
    let jit = compiler::jit::Jit::new(codegen.module())?;

    // For now we assume the entry point is "main"
    let result = jit.run("main")?;

    println!("Result: {}", result);

    Ok(())
}
