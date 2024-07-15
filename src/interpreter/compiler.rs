use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::transmute;
use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RuntimeError, RResult};
use crate::interpreter::chunks::Chunk;
use crate::interpreter::data::{string_to_ptr, Value};
use crate::interpreter::opcode::OpCode;
use crate::interpreter::runtime::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogic, FunctionLogicDescriptor};
use crate::refactor::Refactor;
use crate::refactor::simplify::Simplify;
use crate::transpiler;

pub type InlineFunction = Rc<dyn Fn(&mut FunctionCompiler, &ExpressionID) -> RResult<()>>;

pub struct FunctionCompiler<'a> {
    pub runtime: &'a Runtime,
    pub implementation: &'a FunctionImplementation,
    pub chunk: Chunk,
    pub locals: HashMap<Rc<ObjectReference>, u32>,
    pub constants: Vec<Value>,
}

pub fn compile_deep(runtime: &mut Runtime, function: &Rc<FunctionHead>) -> RResult<Chunk> {
    let FunctionLogic::Implementation(implementation) = runtime.source.fn_logic[function].clone() else {
        return Err(RuntimeError::error("main! function was somehow internal.").to_array());
    };

    let mut refactor = Refactor::new(runtime);
    refactor.add(implementation);

    let mut simplify = Simplify::new(&mut refactor, &transpiler::Config::default());
    simplify.run();

    let needed_functions = refactor.gather_needed_functions();
    let fn_logic = refactor.fn_logic;

    let mut errors = vec![];

    for function in needed_functions {
        match &fn_logic[&function] {
            FunctionLogic::Descriptor(d) => {
                if runtime.function_inlines.contains_key(&function) || runtime.function_evaluators.contains_key(&function.function_id) {
                    continue
                }

                compile_descriptor(&function, d, runtime);
            }
            FunctionLogic::Implementation(implementation) => {
                match compile_function(runtime, implementation) {
                    Ok(compiled) => drop(runtime.function_evaluators.insert(function.function_id, compiled)),
                    Err(err) => errors.extend(err),
                };
            }
        }
    }

    let FunctionLogic::Implementation(implementation) = &fn_logic[function] else {
        errors.push(RuntimeError::error("main! function was somehow internal after refactor."));
        return Err(errors);
    };

    match compile_function(runtime, implementation) {
        Ok(compiled) => {
            if !errors.is_empty() { Err(errors) }
            else { Ok(compiled) }
        },
        Err(err) => {
            errors.extend(err);
            Err(errors)
        },
    }
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

    // println!("{:?}", implementation.head);
    // disassemble(&compiler.chunk);
    // println!("\n");

    Ok(compiler.chunk)
}

impl FunctionCompiler<'_> {
    pub fn compile_expression(&mut self, expression: &ExpressionID) -> RResult<()> {
        let operation = &self.implementation.expression_tree.values[expression];

        match operation {
            ExpressionOperation::Block => {
                let arguments = &self.implementation.expression_tree.children[expression];
                for expr in arguments {
                    self.compile_expression(expr)?;
                    let type_ = &self.implementation.type_forest.resolve_binding_alias(expr)?;
                    if !type_.unit.is_void() {
                        self.chunk.push(OpCode::POP64);
                    }
                }
            },
            ExpressionOperation::GetLocal(local) => {
                let slot = self.get_variable_slot(local);
                self.chunk.push_with_u32(OpCode::LOAD_LOCAL, slot);
            },
            ExpressionOperation::SetLocal(local) => {
                let arguments = &self.implementation.expression_tree.children[expression];
                assert_eq!(arguments.len(), 1);
                self.compile_expression(&arguments[0])?;
                let slot = self.get_variable_slot(local);
                self.chunk.push_with_u32(OpCode::STORE_LOCAL, slot);
            },
            ExpressionOperation::Return => todo!(),
            ExpressionOperation::FunctionCall(function) => {
                if let Some(inline_fn) = self.runtime.function_inlines.get(&function.function) {
                    inline_fn(self, expression);
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
            ExpressionOperation::IfThenElse => {
                let arguments = &self.implementation.expression_tree.children[expression];

                // Condition
                self.compile_expression(&arguments[0])?;

                let jump_location_skip_consequent = self.chunk.code.len();
                self.chunk.push_with_u32(OpCode::JUMP_IF_FALSE, 0);

                // Consequent
                self.compile_expression(&arguments[1])?;
                self.fix_jump_location_i32(jump_location_skip_consequent);

                if let Some(alternative) = arguments.get(2) {
                    let jump_location_skip_alternative = self.chunk.code.len();
                    self.chunk.push_with_u32(OpCode::JUMP, 0);

                    // Alternative
                    self.compile_expression(alternative)?;
                    self.fix_jump_location_i32(jump_location_skip_alternative);
                }
            },
        }

        Ok(())
    }

    pub fn fix_jump_location_i32(&mut self, jump_location: usize) {
        // +5 because opcode and argument were popped
        let distance_skip_consequence = self.chunk.code.len() - (jump_location + 5);
        unsafe {
            self.chunk.modify_u32(jump_location + 1, transmute(i32::try_from(distance_skip_consequence).unwrap()));
        }
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
        FunctionLogicDescriptor::Clone(_) => todo!("{:?}", function),
        FunctionLogicDescriptor::TraitProvider(_) => todo!(),
        FunctionLogicDescriptor::FunctionProvider(f) => {
            let uuid = f.function_id;
            runtime.function_inlines.insert(Rc::clone(function), Rc::new(move |compiler, expression| {
                compiler.chunk.push_with_u128(OpCode::LOAD128, uuid.as_u128());
                Ok(())
            }));
        }
        FunctionLogicDescriptor::PrimitiveOperation { .. } => todo!("{:?}", descriptor),
        FunctionLogicDescriptor::Constructor(_) => todo!(),
        FunctionLogicDescriptor::GetMemberField(_, _) => todo!(),
        FunctionLogicDescriptor::SetMemberField(_, _) => todo!(),
    }
}
