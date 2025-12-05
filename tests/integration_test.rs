use matrix_script::compiler;
use inkwell::context::Context;

#[test]
fn test_math_expression() {
    // The requirement is to feed "3.0 + 2.0" and assert 5.0.
    // Since our language structure requires a function, we wrap it.
    let code_body = "3.0 + 2.0";
    let source = format!("fn main() {{ return {}; }}", code_body);

    let mut parser = compiler::parser::Parser::new(&source).expect("Failed to create parser");
    let program = parser.parse_program().expect("Failed to parse program");

    let context = Context::create();
    let mut codegen = compiler::codegen::CodeGen::new(&context, "test_module");
    codegen.compile_program(&program).expect("Failed to compile program");

    let jit = compiler::jit::Jit::new(codegen.module()).expect("Failed to create JIT");
    let result = jit.run("main").expect("Failed to run main");

    assert_eq!(result, 5.0);
}

#[test]
fn test_complex_math() {
    let source = "
    fn main() {
        let a = 10.0;
        let b = 20.0;
        return a * b + 5.0;
    }
    ";

    let mut parser = compiler::parser::Parser::new(source).expect("Failed to create parser");
    let program = parser.parse_program().expect("Failed to parse program");

    let context = Context::create();
    let mut codegen = compiler::codegen::CodeGen::new(&context, "test_module");
    codegen.compile_program(&program).expect("Failed to compile program");

    let jit = compiler::jit::Jit::new(codegen.module()).expect("Failed to create JIT");
    let result = jit.run("main").expect("Failed to run main");

    assert_eq!(result, 205.0);
}
