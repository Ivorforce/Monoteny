mod builtins;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::{Chunk, Code};
use crate::interpreter::compiler::builtins::compile_builtin_function_call;
use crate::interpreter::data::{bytes_to_stack_slots, get_size_bytes};
use crate::interpreter::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogic};

pub struct FunctionCompiler<'a> {
    runtime: &'a Runtime,
    implementation: &'a FunctionImplementation,
    chunk: Chunk,
    locals: HashMap<Rc<ObjectReference>, u32>,
}

pub fn compile(runtime: &mut Runtime, function: &Rc<FunctionHead>) -> RResult<Chunk> {
    let FunctionLogic::Implementation(implementation) = &runtime.source.fn_logic[function] else {
        return Err(RuntimeError::new("Cannot run function because it is not implemented.".to_string()));
    };

    let mut compiler = FunctionCompiler {
        runtime,
        implementation,
        chunk: Chunk::new(),
        locals: HashMap::new()
    };

    compiler.compile_expression(&implementation.expression_tree.root)?;
    // The root expression is implicitly returned.
    compiler.chunk.push(Code::RETURN);

    compiler.chunk.locals = vec![0; compiler.locals.len()];
    for (obj, idx) in compiler.locals {
        compiler.chunk.locals[usize::try_from(idx).unwrap()] = bytes_to_stack_slots(get_size_bytes(&obj.type_));
    }

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
                    let type_ = &self.implementation.type_forest.resolve_binding_alias(expr)?;
                    let size_slots = bytes_to_stack_slots(get_size_bytes(type_));
                    if size_slots > 0 {
                        todo!("Need to pop")
                    }
                }
            },
            ExpressionOperation::GetLocal(local) => {
                let slot = self.get_variable_slot(local);
                self.chunk.push_with_u32(Code::LOAD_LOCAL, slot);
            },
            ExpressionOperation::SetLocal(local) => {
                let slot = self.get_variable_slot(local);
                self.chunk.push_with_u32(Code::STORE_LOCAL, slot);
            },
            ExpressionOperation::Return => todo!(),
            ExpressionOperation::FunctionCall(function) => {
                for expr in children.iter().rev() {
                    self.compile_expression(expr)?;
                }

                let logic = &self.runtime.source.fn_logic[&function.function];
                match logic {
                    FunctionLogic::Implementation(i) => {
                        if !function.requirements_fulfillment.is_empty() {
                            return Err(RuntimeError::new(format!("Internal error; function call to {:?} was not monomorphized before call.", function.function)));
                        }

                        todo!()
                    }
                    FunctionLogic::Descriptor(d) => {
                        compile_builtin_function_call(d, function, &mut self.chunk, &self.runtime)?;
                    }
                }
            },
            ExpressionOperation::PairwiseOperations { .. } => todo!(),
            ExpressionOperation::ArrayLiteral => todo!(),
            ExpressionOperation::StringLiteral(string) => todo!(),
        }

        Ok(())
    }

    pub fn get_variable_slot(&mut self, object: &Rc<ObjectReference>) -> u32 {
        let count = self.locals.len();

        match self.locals.entry(Rc::clone(object)) {
            Entry::Occupied(o) => *o.get(),
            Entry::Vacant(v) => {
                *v.insert(u32::try_from(count).unwrap())
            }
        }
    }
}

pub fn make_function_getter(function: &FunctionHead) -> Chunk {
    let mut chunk = Chunk::new();
    get_function(function, &mut chunk);
    chunk.push(Code::RETURN);
    chunk
}

fn get_function(function: &FunctionHead, chunk: &mut Chunk) {
    chunk.push_with_u128(Code::LOAD128, function.function_id.as_u128());
}
