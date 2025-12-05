use anyhow::{anyhow, bail, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};
use std::collections::HashMap;

use crate::compiler::ast::{Expr, Function, Op, Program, Stmt};

/// The CodeGen struct which holds the LLVM context, module, and builder.
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
    matrix_type: StructType<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    /// Creates a new CodeGen instance.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        let f64_type = context.f64_type();
        let i64_type = context.i64_type();
        let f64_ptr_type = f64_type.ptr_type(inkwell::AddressSpace::default());
        let matrix_type = context.struct_type(&[f64_ptr_type.into(), i64_type.into(), i64_type.into()], false);

        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            matrix_type,
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
        // Infer return type
        let return_type = self.infer_return_type(function);

        let fn_type = match return_type {
             FunctionReturnType::Matrix => {
                // Return a pointer to the matrix struct
                 self.matrix_type.ptr_type(inkwell::AddressSpace::default()).fn_type(&[], false)
             }
             FunctionReturnType::Scalar => {
                 self.context.f64_type().fn_type(&[], false)
             }
        };

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

    fn infer_return_type(&self, function: &Function) -> FunctionReturnType {
        let mut local_types = HashMap::new();

        for stmt in &function.body {
            match stmt {
                Stmt::Let(name, expr) => {
                    let ty = self.infer_expr_type(expr, &local_types);
                    local_types.insert(name.clone(), ty);
                }
                Stmt::Return(expr) => {
                    return self.infer_expr_type(expr, &local_types);
                }
            }
        }
        FunctionReturnType::Scalar // Default
    }

    fn infer_expr_type(&self, expr: &Expr, locals: &HashMap<String, FunctionReturnType>) -> FunctionReturnType {
        match expr {
            Expr::Number(_) => FunctionReturnType::Scalar,
            Expr::MatrixLiteral(_) => FunctionReturnType::Matrix,
            Expr::Identifier(name) => *locals.get(name).unwrap_or(&FunctionReturnType::Scalar),
            Expr::BinaryOp(left, _, right) => {
                let lhs = self.infer_expr_type(left, locals);
                let rhs = self.infer_expr_type(right, locals);
                if lhs == FunctionReturnType::Matrix || rhs == FunctionReturnType::Matrix {
                    FunctionReturnType::Matrix
                } else {
                    FunctionReturnType::Scalar
                }
            }
        }
    }

    /// Compiles a statement.
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let(name, expr) => {
                let val = self.compile_expr(expr)?;
                let ty = val.get_type();
                // Create alloca
                let alloca = self.create_entry_block_alloca(name, ty);
                self.builder.build_store(alloca, val)?;
                self.variables.insert(name.clone(), (alloca, ty));
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
                    Some((ptr, ty)) => {
                         let val = self.builder.build_load(*ty, *ptr, name)?;
                         Ok(val)
                    }
                    None => bail!("Variable not found: {}", name),
                }
            }
            Expr::BinaryOp(left, op, right) => {
                let lhs = self.compile_expr(left)?;
                let rhs = self.compile_expr(right)?;

                // Check types
                if lhs.is_float_value() && rhs.is_float_value() {
                    let lhs_float = lhs.into_float_value();
                    let rhs_float = rhs.into_float_value();
                     let res = match op {
                        Op::Add => self.builder.build_float_add(lhs_float, rhs_float, "addtmp")?,
                        Op::Subtract => self.builder.build_float_sub(lhs_float, rhs_float, "subtmp")?,
                        Op::Multiply => self.builder.build_float_mul(lhs_float, rhs_float, "multmp")?,
                        Op::Divide => self.builder.build_float_div(lhs_float, rhs_float, "divtmp")?,
                    };
                    Ok(res.into())
                } else if lhs.is_pointer_value() && rhs.is_pointer_value() {
                     // Matrix + Matrix
                     match op {
                         Op::Add => self.compile_matrix_add(lhs.into_pointer_value(), rhs.into_pointer_value()),
                         _ => bail!("Operator {:?} not supported for matrices yet", op),
                     }
                } else {
                    bail!("Type mismatch in binary operation")
                }
            }
            Expr::MatrixLiteral(rows) => {
                self.compile_matrix_literal(rows)
            }
        }
    }

    fn compile_matrix_literal(&mut self, rows: &Vec<Vec<Expr>>) -> Result<BasicValueEnum<'ctx>> {
         let num_rows = rows.len() as u64;
         if num_rows == 0 {
             bail!("Empty matrix literal");
         }
         let num_cols = rows[0].len() as u64;

         // Verify all rows have same length
         for row in rows {
             if row.len() as u64 != num_cols {
                 bail!("Matrix rows must have same length");
             }
         }

         let total_size = num_rows * num_cols;

         // Allocate data array: double* data = malloc(total_size * sizeof(double))
         let f64_type = self.context.f64_type();
         let i64_type = self.context.i64_type();
         let total_size_val = i64_type.const_int(total_size, false);

         // We need to call malloc. Inkwell's `build_array_malloc` usually expects the type being allocated.
         let data_ptr = self.builder.build_array_malloc(f64_type, total_size_val, "matrix_data")?;

         // Populate data
         for (i, row) in rows.iter().enumerate() {
             for (j, expr) in row.iter().enumerate() {
                 let val = self.compile_expr(expr)?;
                 if !val.is_float_value() {
                     bail!("Matrix elements must be numbers");
                 }
                 let float_val = val.into_float_value();

                 // index = i * cols + j
                 let index = i as u64 * num_cols + j as u64;
                 let index_val = i64_type.const_int(index, false);

                 // GEP
                 unsafe {
                    let ptr = self.builder.build_gep(f64_type, data_ptr, &[index_val], "elem_ptr")?;
                    self.builder.build_store(ptr, float_val)?;
                 }
             }
         }

         // Create Matrix struct
         let matrix_ptr = self.builder.build_malloc(self.matrix_type, "matrix_struct")?;

         // Store data ptr
         let data_field_ptr = self.builder.build_struct_gep(self.matrix_type, matrix_ptr, 0, "data_field")
            .map_err(|_| anyhow!("Struct GEP failed"))?;
         self.builder.build_store(data_field_ptr, data_ptr)?;

         // Store rows
         let rows_field_ptr = self.builder.build_struct_gep(self.matrix_type, matrix_ptr, 1, "rows_field")
            .map_err(|_| anyhow!("Struct GEP failed"))?;
         self.builder.build_store(rows_field_ptr, i64_type.const_int(num_rows, false))?;

         // Store cols
         let cols_field_ptr = self.builder.build_struct_gep(self.matrix_type, matrix_ptr, 2, "cols_field")
             .map_err(|_| anyhow!("Struct GEP failed"))?;
         self.builder.build_store(cols_field_ptr, i64_type.const_int(num_cols, false))?;

         Ok(matrix_ptr.into())
    }

    fn compile_matrix_add(&mut self, lhs_ptr: PointerValue<'ctx>, rhs_ptr: PointerValue<'ctx>) -> Result<BasicValueEnum<'ctx>> {
        let i64_type = self.context.i64_type();
        let f64_type = self.context.f64_type();

        // Load dimensions from LHS (Assume LHS and RHS dimensions match for now)
        let rows_ptr = self.builder.build_struct_gep(self.matrix_type, lhs_ptr, 1, "rows_ptr")
            .map_err(|_| anyhow!("GEP failed"))?;
        let rows = self.builder.build_load(i64_type, rows_ptr, "rows")?.into_int_value();

        let cols_ptr = self.builder.build_struct_gep(self.matrix_type, lhs_ptr, 2, "cols_ptr")
             .map_err(|_| anyhow!("GEP failed"))?;
        let cols = self.builder.build_load(i64_type, cols_ptr, "cols")?.into_int_value();

        let total_size = self.builder.build_int_mul(rows, cols, "total_size");

        // Allocate result data
        let res_data_ptr = self.builder.build_array_malloc(f64_type, total_size, "res_data")?;

        // Get data pointers
        let lhs_data_ptr_ptr = self.builder.build_struct_gep(self.matrix_type, lhs_ptr, 0, "lhs_data_ptr")
            .map_err(|_| anyhow!("GEP failed"))?;
        let lhs_data_ptr = self.builder.build_load(f64_type.ptr_type(inkwell::AddressSpace::default()), lhs_data_ptr_ptr, "lhs_data")?.into_pointer_value();

        let rhs_data_ptr_ptr = self.builder.build_struct_gep(self.matrix_type, rhs_ptr, 0, "rhs_data_ptr")
             .map_err(|_| anyhow!("GEP failed"))?;
        let rhs_data_ptr = self.builder.build_load(f64_type.ptr_type(inkwell::AddressSpace::default()), rhs_data_ptr_ptr, "rhs_data")?.into_pointer_value();

        // Loop
        let loop_block = self.context.append_basic_block(self.builder.get_insert_block().unwrap().get_parent().unwrap(), "loop");
        let after_block = self.context.append_basic_block(self.builder.get_insert_block().unwrap().get_parent().unwrap(), "after_loop");

        // Incoming value for phi requires predecessors.
        // We need to branch to loop_block from current block.
        let entry_block = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(loop_block)?;

        self.builder.position_at_end(loop_block);

        let i = self.builder.build_phi(i64_type, "i")?;
        // We will add incoming values later? No, we need to do it now or update it.
        // i comes from entry (0) or loop (next_i).
        i.add_incoming(&[(&i64_type.const_int(0, false), entry_block)]);

        // Loop body
        // Load A[i]
        let lhs_elem_ptr = unsafe { self.builder.build_gep(f64_type, lhs_data_ptr, &[i.as_basic_value().into_int_value()], "lhs_elem_ptr")? };
        let lhs_val = self.builder.build_load(f64_type, lhs_elem_ptr, "lhs_val")?.into_float_value();

        // Load B[i]
        let rhs_elem_ptr = unsafe { self.builder.build_gep(f64_type, rhs_data_ptr, &[i.as_basic_value().into_int_value()], "rhs_elem_ptr")? };
        let rhs_val = self.builder.build_load(f64_type, rhs_elem_ptr, "rhs_val")?.into_float_value();

        // Add
        let res_val = self.builder.build_float_add(lhs_val, rhs_val, "sum");

        // Store Result[i]
        let res_elem_ptr = unsafe { self.builder.build_gep(f64_type, res_data_ptr, &[i.as_basic_value().into_int_value()], "res_elem_ptr")? };
        self.builder.build_store(res_elem_ptr, res_val)?;

        // Increment
        let next_i = self.builder.build_int_add(i.as_basic_value().into_int_value(), i64_type.const_int(1, false), "next_i");
        // Loop back
        i.add_incoming(&[(&next_i, loop_block)]);

        // Condition
        let cmp = self.builder.build_int_compare(inkwell::IntPredicate::SLT, next_i, total_size, "cmp");
        self.builder.build_conditional_branch(cmp, loop_block, after_block)?;

        self.builder.position_at_end(after_block);

        // Create Result Struct
        let res_matrix_ptr = self.builder.build_malloc(self.matrix_type, "res_matrix")?;

        // Set data
        let res_data_field = self.builder.build_struct_gep(self.matrix_type, res_matrix_ptr, 0, "res_data_field")
            .map_err(|_| anyhow!("GEP failed"))?;
        self.builder.build_store(res_data_field, res_data_ptr)?;

        // Set rows
        let res_rows_field = self.builder.build_struct_gep(self.matrix_type, res_matrix_ptr, 1, "res_rows_field")
             .map_err(|_| anyhow!("GEP failed"))?;
        self.builder.build_store(res_rows_field, rows)?;

        // Set cols
        let res_cols_field = self.builder.build_struct_gep(self.matrix_type, res_matrix_ptr, 2, "res_cols_field")
             .map_err(|_| anyhow!("GEP failed"))?;
        self.builder.build_store(res_cols_field, cols)?;

        Ok(res_matrix_ptr.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionReturnType {
    Scalar,
    Matrix,
}
