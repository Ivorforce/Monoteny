use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::Chunk;
use crate::interpreter::compile::function_compiler::{compile_function, InlineFunction};
use crate::interpreter::compile::function_descriptor_compiler::compile_descriptor;
use crate::interpreter::data_layout::{create_data_layout, DataLayout};
use crate::program::functions::{FunctionHead, FunctionLogic};
use crate::program::traits::StructInfo;
use crate::refactor::simplify::Simplify;
use crate::refactor::Refactor;
use crate::source::Source;

pub struct CompileServer {
    pub simplify: Simplify,

    // These are optimized for running and may not reflect the source code itself.
    // They are also only loaded on demand.
    pub function_evaluators: HashMap<Uuid, Rc<Chunk>>,
    pub function_inlines: HashMap<Uuid, InlineFunction>,
    // TODO We'll need these only in the future when we compile functions to constants.
    // pub global_assignments: HashMap<Uuid, Value>,
    pub data_layouts: HashMap<Rc<StructInfo>, Rc<DataLayout>>,
}

impl CompileServer {
    pub fn new() -> CompileServer {
        CompileServer {
            simplify: Simplify {
                refactor: Refactor::new(),
                inline: true,
                trim_locals: true,
                monomorphize: true,
            },
            function_evaluators: Default::default(),
            function_inlines: Default::default(),
            data_layouts: Default::default(),
        }
    }

    pub fn compile_deep(&mut self, source: &Source, function_head: &Rc<FunctionHead>) -> RResult<Rc<Chunk>> {
        // Let's see if it's compiled already.

        if let Some(evaluator) = self.function_evaluators.get(&function_head.function_id) {
            return Ok(Rc::clone(evaluator))
        }

        // Our function was inlined
        if let Some(inline) = self.function_inlines.get(&function_head.function_id) {
            todo!("We can still (probably) compile the inline to an implementation, and therefore chunk!")
        }

        // If not - let's compile it!
        match &source.fn_logic[function_head] {
            FunctionLogic::Descriptor(d) => {
                // Okay, kinda awkward. Someone wants to compile a descriptor.
                compile_descriptor(&function_head, &d, self);

                // Ok, it worked. We should have either a function evaluator or an inline.
                if let Some(evaluator) = self.function_evaluators.get(&function_head.function_id) {
                    return Ok(Rc::clone(evaluator))
                }

                if let Some(inline) = self.function_inlines.get(&function_head.function_id) {
                    todo!("We can still (probably) compile the inline to an implementation, and therefore chunk!")
                }

                unreachable!()
            },
            FunctionLogic::Implementation(implementation) => {
                self.simplify.refactor.add(Rc::clone(function_head), implementation.clone());

                self.simplify.run([function_head].into_iter(), source);

                let needed_functions = self.simplify.refactor.gather_deep_functions([function_head].into_iter(), source);

                let mut errors = vec![];

                for function in needed_functions {
                    // FIXME We shouldn't clone it here... But compile_descriptor needs a mutable ref to self
                    //  because it needs to insert inlines AND data layouts. Which means the mutable ref can
                    //  get invalidated.
                    match self.simplify.refactor.fn_logic[&function].clone() {
                        FunctionLogic::Descriptor(d) => {
                            if self.function_inlines.contains_key(&function.function_id) || self.function_evaluators.contains_key(&function.function_id) {
                                continue
                            }

                            compile_descriptor(&function, &d, self);
                        }
                        FunctionLogic::Implementation(implementation) => {
                            match compile_function(self, &implementation) {
                                Ok(compiled) => drop(self.function_evaluators.insert(function.function_id, compiled)),
                                Err(err) => errors.extend(err),
                            };
                        }
                    }
                }

                let FunctionLogic::Implementation(implementation) = &self.simplify.refactor.fn_logic[function_head] else {
                    errors.push(RuntimeError::error("main! function was somehow internal after refactor."));
                    return Err(errors);
                };

                match compile_function(self, implementation) {
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
        }
    }

    pub fn compile_function(&mut self, function: &Rc<FunctionHead>) -> RResult<()> {
        match self.simplify.refactor.fn_logic[function].clone() {
            FunctionLogic::Descriptor(d) => {
                if self.function_inlines.contains_key(&function.function_id) || self.function_evaluators.contains_key(&function.function_id) {
                    return Ok(())
                }

                compile_descriptor(&function, &d, self);
            }
            FunctionLogic::Implementation(implementation) => {
                let compiled = compile_function(self, &implementation)?;
                self.function_evaluators.insert(function.function_id, compiled);
            }
        }

        return Ok(())
    }

    pub fn get_data_layout(&mut self, struct_info: &Rc<StructInfo>) -> Rc<DataLayout> {
        if let Some(layout) = self.data_layouts.get(struct_info) {
            return Rc::clone(layout)
        }

        let layout = create_data_layout(Rc::clone(struct_info));
        self.data_layouts.insert(Rc::clone(struct_info), Rc::clone(&layout));
        layout
    }
}
