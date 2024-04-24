use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::Chunk;
use crate::interpreter::data::{bytes_to_stack_slots, get_size_bytes, string_to_ptr, Value};
use crate::interpreter::opcode::OpCode;
use crate::interpreter::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogic, FunctionLogicDescriptor};
use crate::refactor::Refactor;
use crate::refactor::simplify::Simplify;
use crate::transpiler;

pub type InlineFunction = Rc<dyn Fn(&mut FunctionCompiler)>;

pub struct FunctionCompiler<'a> {
    pub runtime: &'a Runtime,
    pub implementation: &'a FunctionImplementation,
    pub chunk: Chunk,
    pub locals: HashMap<Rc<ObjectReference>, u32>,
    pub constants: Vec<Value>,
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

    let needed_functions = refactor.gather_needed_functions();
    let fn_logic = refactor.fn_logic;

    for function in needed_functions {
        match &fn_logic[&function] {
            FunctionLogic::Descriptor(d) => {
                if runtime.function_inlines.contains_key(&function) || runtime.function_evaluators.contains_key(&function.function_id) {
                    continue
                }

                compile_descriptor(&function, d, runtime);
            }
            FunctionLogic::Implementation(implementation) => {
                let compiled = compile_function(runtime, implementation)?;
                runtime.function_evaluators.insert(function.function_id, compiled);
            }
        }
    }

    let FunctionLogic::Implementation(implementation) = &fn_logic[function] else {
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

                if let Some(inline_fn) = self.runtime.function_inlines.get(&function.function) {
                    inline_fn(self);
                }
                else {
                    todo!()
                }
            },
            ExpressionOperation::PairwiseOperations { .. } => todo!(),
            ExpressionOperation::ArrayLiteral => todo!(),
            ExpressionOperation::StringLiteral(string) => {
                unsafe {
                    self.constants.push(Value { ptr: string_to_ptr(string) });
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

pub fn compile_descriptor(function: &Rc<FunctionHead>, descriptor: &FunctionLogicDescriptor, runtime: &mut Runtime) {
    match descriptor {
        FunctionLogicDescriptor::Stub => todo!("{:?}", function),
        FunctionLogicDescriptor::TraitProvider(_) => todo!(),
        FunctionLogicDescriptor::FunctionProvider(f) => {
            let uuid = f.function_id;
            runtime.function_inlines.insert(Rc::clone(function), Rc::new(move |compiler| {
                compiler.chunk.push_with_u128(OpCode::LOAD128, uuid.as_u128());
            }));
        }
        FunctionLogicDescriptor::PrimitiveOperation { .. } => todo!("{:?}", descriptor),
        FunctionLogicDescriptor::Constructor(_) => todo!(),
        FunctionLogicDescriptor::GetMemberField(_, _) => todo!(),
        FunctionLogicDescriptor::SetMemberField(_, _) => todo!(),
    }
}
