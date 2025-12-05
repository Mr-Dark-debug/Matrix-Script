use anyhow::{anyhow, bail, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};
use std::collections::HashMap;

use crate::compiler::ast::{Expr, Function, Op, Program, Stmt};

/// The CodeGen struct which holds the LLVM context, module, and builder.
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    /// Creates a new CodeGen instance.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    /// Returns a reference to the inner module.
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Compiles a program.
    pub fn compile_program(&mut self, program: &Program) -> Result<()> {
        for function in &program.functions {
            self.compile_function(function)?;
        }
        Ok(())
    }

    /// Compiles a function.
    fn compile_function(&mut self, function: &Function) -> Result<()> {
        // Define function type: fn() -> f64 (assuming main returns f64 for now as per example)
        // actually example returns `a * b + 5.0` which is f64.
        let f64_type = self.context.f64_type();
        let fn_type = f64_type.fn_type(&[], false);
        let fn_val = self.module.add_function(&function.name, fn_type, None);

        // Create basic block
        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        // Clear variables for new function scope
        self.variables.clear();

        for stmt in &function.body {
            self.compile_stmt(stmt)?;
        }

        Ok(())
    }

    /// Compiles a statement.
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let(name, expr) => {
                let val = self.compile_expr(expr)?;
                // Create alloca
                let alloca = self.create_entry_block_alloca(name, val.get_type());
                self.builder.build_store(alloca, val)?;
                self.variables.insert(name.clone(), alloca);
                Ok(())
            }
            Stmt::Return(expr) => {
                let val = self.compile_expr(expr)?;
                self.builder.build_return(Some(&val))?;
                Ok(())
            }
        }
    }

    /// Helper to create alloca in the entry block.
    fn create_entry_block_alloca(&self, name: &str, ty: BasicTypeEnum<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self.builder.get_insert_block().unwrap().get_parent().unwrap().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(ty, name).unwrap()
    }

    /// Compiles an expression.
    fn compile_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Number(n) => Ok(self.context.f64_type().const_float(*n).into()),
            Expr::Identifier(name) => {
                match self.variables.get(name) {
                    Some(ptr) => {
                        let val = self.builder.build_load(self.context.f64_type(), *ptr, name)?;
                        Ok(val)
                    }
                    None => bail!("Variable not found: {}", name),
                }
            }
            Expr::BinaryOp(left, op, right) => {
                let lhs = self.compile_expr(left)?.into_float_value();
                let rhs = self.compile_expr(right)?.into_float_value();

                let res = match op {
                    Op::Add => self.builder.build_float_add(lhs, rhs, "addtmp")?,
                    Op::Subtract => self.builder.build_float_sub(lhs, rhs, "subtmp")?,
                    Op::Multiply => self.builder.build_float_mul(lhs, rhs, "multmp")?,
                    Op::Divide => self.builder.build_float_div(lhs, rhs, "divtmp")?,
                };
                Ok(res.into())
            }
            Expr::MatrixLiteral(_) => {
                // TODO: Implement Matrix construction
                // Phase 2: Matrix SIMD.
                // Setup struct wrapper type but return error for now as we deal with scalars in Phase 1 MVP.
                let _matrix_type = self.get_matrix_type();
                bail!("Matrix literals not yet supported in codegen phase 1")
            }
        }
    }

    /// Defines the matrix struct type: { double* data, i64 rows, i64 cols }
    fn get_matrix_type(&self) -> inkwell::types::StructType<'ctx> {
        let f64_type = self.context.f64_type();
        let i64_type = self.context.i64_type();
        let f64_ptr_type = f64_type.ptr_type(inkwell::AddressSpace::default());
        self.context.struct_type(&[f64_ptr_type.into(), i64_type.into(), i64_type.into()], false)
    }
}
