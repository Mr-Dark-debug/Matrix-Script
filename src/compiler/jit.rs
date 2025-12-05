use anyhow::{anyhow, Result};
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::OptimizationLevel;

/// The JIT engine.
pub struct Jit<'ctx> {
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Jit<'ctx> {
    /// Creates a new JIT engine for the given module.
    pub fn new(module: &Module<'ctx>) -> Result<Self> {
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| anyhow!("Failed to create execution engine: {}", e))?;
        Ok(Self { execution_engine })
    }

    /// Runs the function with the given name.
    /// Assumes the function takes no arguments and returns f64.
    pub fn run(&self, function_name: &str) -> Result<f64> {
        unsafe {
            let func: inkwell::execution_engine::JitFunction<unsafe extern "C" fn() -> f64> =
                self.execution_engine
                .get_function(function_name)
                .map_err(|_| anyhow!("Function {} not found in JIT", function_name))?;

            Ok(func.call())
        }
    }
}
