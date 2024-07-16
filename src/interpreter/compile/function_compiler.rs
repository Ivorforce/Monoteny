use std::mem::transmute;
use std::rc::Rc;

use itertools::Itertools;

use crate::error::RResult;
use crate::interpreter::chunks::Chunk;
use crate::interpreter::compile::compile_server::CompileServer;
use crate::interpreter::data::{string_to_ptr, Value};
use crate::interpreter::opcode::OpCode;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionImplementation;

pub type InlineFunction = Rc<dyn Fn(&mut FunctionCompiler, &ExpressionID) -> RResult<()>>;

pub struct FunctionCompiler<'a> {
    pub compile_server: &'a CompileServer,
    pub implementation: &'a FunctionImplementation,
    pub chunk: Chunk,
    pub alloced_locals: Vec<Rc<ObjectReference>>,
}

pub fn compile_function(compile_server: &CompileServer, implementation: &FunctionImplementation) -> RResult<Rc<Chunk>> {
    let mut compiler = FunctionCompiler {
        compile_server,
        implementation,
        chunk: Chunk::new(),
        alloced_locals: vec![],
    };

    // For now, they have an arbitrary order.
    let locals = implementation.locals_names.keys()
        .filter(|l| !implementation.parameter_locals.contains(l))
        .cloned()
        .collect_vec();

    // Parameters are already on the stack when our function is called.
    for _ in locals.iter() {
        compiler.chunk.push(OpCode::LOAD0);
    }
    compiler.alloced_locals.extend(implementation.parameter_locals.clone());
    compiler.alloced_locals.extend(locals);

    // Compile the main expression.
    compiler.compile_expression(&implementation.expression_tree.root)?;

    // Implicit return at the end.
    compiler.compile_return();

    // println!("{:?}", implementation.head);
    // disassemble(&compiler.chunk);
    // println!("\n");

    Ok(Rc::new(compiler.chunk))
}

impl FunctionCompiler<'_> {
    pub fn compile_return(&mut self) {
        // TODO If any of these were allocated, we need to deallocate them.
        // Clean up all locals that are currently allocated.
        for _ in self.alloced_locals.iter().rev() {
            if !self.implementation.head.interface.return_type.unit.is_void() {
                self.chunk.push(OpCode::SWAP64);
            }
            self.chunk.push(OpCode::POP64);
        }

        self.chunk.push(OpCode::RETURN);
    }

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
                unsafe { self.chunk.push_with_u32(OpCode::LOAD_LOCAL_32, transmute(slot)); }
            },
            ExpressionOperation::SetLocal(local) => {
                let arguments = &self.implementation.expression_tree.children[expression];
                assert_eq!(arguments.len(), 1);
                self.compile_expression(&arguments[0])?;
                let slot = self.get_variable_slot(local);
                unsafe { self.chunk.push_with_u32(OpCode::STORE_LOCAL_32, transmute(slot)); }
            },
            ExpressionOperation::Return => {
                // FIXME Need to clean up
                let arguments = &self.implementation.expression_tree.children[expression];
                match &arguments[..] {
                    [arg] => self.compile_expression(arg)?,
                    [] => {},
                    _ => unreachable!(),
                }

                self.compile_return();
            },
            ExpressionOperation::FunctionCall(function_binding) => {
                if let Some(inline_fn) = self.compile_server.function_inlines.get(&function_binding.function) {
                    inline_fn(self, expression)?;
                }
                else {
                    let arguments = &self.implementation.expression_tree.children[expression];
                    for argument in arguments.iter() { self.compile_expression(argument)? }

                    self.chunk.push_with_u128(OpCode::CALL, function_binding.function.function_id.as_u128());
                }
            }
            ExpressionOperation::PairwiseOperations { .. } => todo!(),
            ExpressionOperation::ArrayLiteral => todo!(),
            ExpressionOperation::StringLiteral(string) => {
                unsafe {
                    self.chunk.constants.push(Value { ptr: string_to_ptr(string) });
                    self.chunk.push_with_u32(OpCode::LOAD_CONSTANT_32, u32::try_from(self.chunk.constants.len() - 1).unwrap());
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

    pub fn get_variable_slot(&mut self, object: &Rc<ObjectReference>) -> i32 {
        // Later, locals will be on the stack and dynamically added and removed.
        // For now, all locals are allocated at function entry and deallocated at function exit.
        i32::try_from(self.alloced_locals.iter().position(|l| l == object).unwrap()).unwrap() - i32::try_from(self.implementation.head.interface.parameters.len()).unwrap()
    }
}
