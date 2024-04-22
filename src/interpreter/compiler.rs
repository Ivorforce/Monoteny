mod builtins;

use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::Chunk;
use crate::interpreter::compiler::builtins::compile_builtin_function;
use crate::interpreter::Runtime;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogic};

pub struct FunctionCompiler<'a> {
    runtime: &'a Runtime,
    implementation: &'a FunctionImplementation,
}

pub fn compile(runtime: &mut Runtime, function: &Rc<FunctionHead>) -> RResult<Chunk> {
    let FunctionLogic::Implementation(implementation) = &runtime.source.fn_logic[function] else {
        return Err(RuntimeError::new("Cannot run function because it is not implemented.".to_string()));
    };

    let mut compiler = FunctionCompiler {
        runtime,
        implementation,
    };
    compiler.compile_expression(&implementation.expression_tree.root)
}

impl FunctionCompiler<'_> {
    pub fn compile_expression(&mut self, expression: &ExpressionID) -> RResult<Chunk> {
        let operation = &self.implementation.expression_tree.values[expression];
        let children = &self.implementation.expression_tree.children[expression];

        match operation {
            ExpressionOperation::Block => {
                todo!()
            },
            ExpressionOperation::GetLocal(local) => todo!(),
            ExpressionOperation::SetLocal(local) => todo!(),
            ExpressionOperation::Return => todo!(),
            ExpressionOperation::FunctionCall(function) => {
                if !function.requirements_fulfillment.is_empty() {
                    return Err(RuntimeError::new("Internal error; function was not monomorphized before call.".to_string()));
                }

                let logic = &self.runtime.source.fn_logic[&function.function];
                match logic {
                    FunctionLogic::Implementation(i) => {
                        todo!()
                    }
                    FunctionLogic::Descriptor(d) => {
                        compile_builtin_function(d)
                    }
                }
            },
            ExpressionOperation::PairwiseOperations { .. } => todo!(),
            ExpressionOperation::ArrayLiteral => todo!(),
            ExpressionOperation::StringLiteral(string) => todo!(),
        }
    }
}

pub fn make_function_getter(function: &FunctionHead) -> Chunk {
    todo!()
}
