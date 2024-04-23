mod builtins;

use std::alloc::{alloc, Layout};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::transmute;
use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::{Chunk, OpCode};
use crate::interpreter::compiler::builtins::compile_builtin_function_call;
use crate::interpreter::data::{bytes_to_stack_slots, get_size_bytes, Value};
use crate::interpreter::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogic};
use crate::refactor::Refactor;
use crate::refactor::simplify::Simplify;
use crate::transpiler;

pub struct FunctionCompiler<'a> {
    runtime: &'a Runtime,
    implementation: &'a FunctionImplementation,
    chunk: Chunk,
    locals: HashMap<Rc<ObjectReference>, u32>,
    constants: Vec<Value>,
}

pub fn compile_deep(runtime: &mut Runtime, function: &Rc<FunctionHead>) -> RResult<Chunk> {
    let FunctionLogic::Implementation(implementation) = runtime.source.fn_logic[function].clone() else {
        return Err(RuntimeError::new(format!("main! function was somehow internal.")));
    };
    let function_representation = runtime.source.fn_representations[function].clone();

    let mut refactor = Refactor::new(runtime);
    refactor.add(implementation, function_representation);

    let mut simplify = Simplify::new(&mut refactor, &transpiler::Config::default());
    simplify.run();

    // TODO We should gather all invented functions, register them in the runtime, and compile them.
    let FunctionLogic::Implementation(implementation) = &refactor.fn_logic[function] else {
        return Err(RuntimeError::new(format!("main! function was somehow internal after refactor.")));
    };

    compile_function(runtime, implementation)
}

fn compile_function(runtime: &mut Runtime, implementation: &FunctionImplementation) -> RResult<Chunk> {
    let mut compiler = FunctionCompiler {
        runtime,
        implementation,
        chunk: Chunk::new(),
        locals: HashMap::new(),
        constants: vec![],
    };

    compiler.compile_expression(&implementation.expression_tree.root)?;
    // The root expression is implicitly returned.
    compiler.chunk.push(OpCode::RETURN);

    compiler.chunk.locals_count = u32::try_from(compiler.locals.len()).unwrap();
    compiler.chunk.constants = compiler.constants;

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
                self.chunk.push_with_u32(OpCode::LOAD_LOCAL, slot);
            },
            ExpressionOperation::SetLocal(local) => {
                let slot = self.get_variable_slot(local);
                self.chunk.push_with_u32(OpCode::STORE_LOCAL, slot);
            },
            ExpressionOperation::Return => todo!(),
            ExpressionOperation::FunctionCall(function) => {
                for expr in children.iter() {
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
            ExpressionOperation::StringLiteral(string) => {
                unsafe {
                    let data = alloc(Layout::new::<String>());
                    *(data as *mut String) = string.clone();
                    self.constants.push(Value { ptr: transmute(data) });
                    self.chunk.push_with_u32(OpCode::LOAD_CONSTANT, u32::try_from(self.constants.len() - 1).unwrap());
                }
            },
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
    chunk.push(OpCode::RETURN);
    chunk
}

fn get_function(function: &FunctionHead, chunk: &mut Chunk) {
    chunk.push_with_u128(OpCode::LOAD128, function.function_id.as_u128());
}
