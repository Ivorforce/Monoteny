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
use crate::refactor::Refactor;
use crate::refactor::simplify::Simplify;
use crate::source::Source;

pub struct CompileServer {
    pub simplify: Simplify,

    // These are optimized for running and may not reflect the source code itself.
    // They are also only loaded on demand.
    pub function_evaluators: HashMap<Uuid, Rc<Chunk>>,
    // TODO We'll need these only in the future when we compile functions to constants.
    // pub global_assignments: HashMap<Uuid, Value>,
    pub function_inlines: HashMap<Rc<FunctionHead>, InlineFunction>,
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

    pub fn compile_deep(&mut self, source: &Source, function: &Rc<FunctionHead>) -> RResult<Rc<Chunk>> {
        let FunctionLogic::Implementation(implementation) = source.fn_logic[function].clone() else {
            return Err(RuntimeError::error("main! function was somehow internal.").to_array());
        };

        self.simplify.refactor.add(implementation);

        self.simplify.run(source);

        let needed_functions = self.simplify.refactor.gather_needed_functions(source);

        let mut errors = vec![];

        for function in needed_functions {
            // FIXME We shouldn't clone it here... But compile_descriptor needs a mutable ref to self
            //  because it needs to insert inlines AND data layouts. Which means the mutable ref can
            //  get invalidated.
            match self.simplify.refactor.fn_logic[&function].clone() {
                FunctionLogic::Descriptor(d) => {
                    if self.function_inlines.contains_key(&function) || self.function_evaluators.contains_key(&function.function_id) {
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

        let FunctionLogic::Implementation(implementation) = &self.simplify.refactor.fn_logic[function] else {
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

    pub fn get_data_layout(&mut self, struct_info: &Rc<StructInfo>) -> Rc<DataLayout> {
        if let Some(layout) = self.data_layouts.get(struct_info) {
            return Rc::clone(layout)
        }

        let layout = create_data_layout(Rc::clone(struct_info));
        self.data_layouts.insert(Rc::clone(struct_info), Rc::clone(&layout));
        layout
    }
}
