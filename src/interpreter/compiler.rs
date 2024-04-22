mod builtins;

use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::{Chunk, Code, Primitive};
use crate::interpreter::compiler::builtins::compile_builtin_function;
use crate::interpreter::Runtime;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogic};

pub struct FunctionCompiler<'a> {
    runtime: &'a Runtime,
    implementation: &'a FunctionImplementation,
    chunk: Chunk,
}

pub fn compile(runtime: &mut Runtime, function: &Rc<FunctionHead>) -> RResult<Chunk> {
    let FunctionLogic::Implementation(implementation) = &runtime.source.fn_logic[function] else {
        return Err(RuntimeError::new("Cannot run function because it is not implemented.".to_string()));
    };

    let mut compiler = FunctionCompiler {
        runtime,
        implementation,
        chunk: Chunk::new()
    };
    compiler.compile_expression(&implementation.expression_tree.root)?;
    Ok(compiler.chunk)
}

impl FunctionCompiler<'_> {
    pub fn compile_expression(&mut self, expression: &ExpressionID) -> RResult<()> {
        let operation = &self.implementation.expression_tree.values[expression];
        let children = &self.implementation.expression_tree.children[expression];

        match operation {
            ExpressionOperation::Block => {
                for expr in children {
                    self.compile_expression(expr)?;
                }
            },
            ExpressionOperation::GetLocal(local) => todo!(),
            ExpressionOperation::SetLocal(local) => todo!(),
            ExpressionOperation::Return => todo!(),
            ExpressionOperation::FunctionCall(function) => {
                if !function.requirements_fulfillment.is_empty() {
                    return Err(RuntimeError::new(format!("Internal error; function call to {:?} was not monomorphized before call.", function.function)));
                }

                let logic = &self.runtime.source.fn_logic[&function.function];
                match logic {
                    FunctionLogic::Implementation(i) => {
                        todo!()
                    }
                    FunctionLogic::Descriptor(d) => {
                        compile_builtin_function(d, &mut self.chunk)?;
                    }
                }
            },
            ExpressionOperation::PairwiseOperations { .. } => todo!(),
            ExpressionOperation::ArrayLiteral => todo!(),
            ExpressionOperation::StringLiteral(string) => todo!(),
        }

        Ok(())
    }

}

pub fn make_function_getter(function: &FunctionHead) -> Chunk {
    let mut chunk = Chunk::new();
    let u64s = function.function_id.as_u64_pair();
    chunk.push_with_u64(Code::LOAD64, u64s.0);
    chunk.push_with_u64(Code::LOAD64, u64s.1);
    chunk.push(Code::RETURN);
    chunk
}
