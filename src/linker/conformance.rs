use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::generic_unfolding::map_interface_types;
use crate::linker::{LinkError, scopes};
use crate::linker::global::FunctionWithoutBody;
use crate::linker::interface::link_function_pointer;
use crate::linker::scopes::Scope;
use crate::parser::abstract_syntax;
use crate::program::allocation::ObjectReference;
use crate::program::builtins::Builtins;
use crate::program::functions::{Function, FunctionCallType, FunctionForm, FunctionPointer};
use crate::program::module::Module;
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};

pub struct ConformanceLinker<'a> {
    pub binding: Rc<TraitBinding>,

    pub builtins: &'a Builtins,

    pub functions: Vec<FunctionWithoutBody<'a>>,
}

impl <'a> ConformanceLinker<'a> {
    pub fn link_statement(&mut self, statement: &'a abstract_syntax::GlobalStatement, requirements: &HashSet<Rc<TraitBinding>>, scope: &scopes::Scope) -> Result<(), LinkError> {
        match statement {
            abstract_syntax::GlobalStatement::FunctionDeclaration(syntax) => {
                let fun = link_function_pointer(&syntax, &scope, requirements)?;
                guard!(let Some(body) = &syntax.body else {
                    return Err(LinkError::LinkError { msg: format!("Function {} needs a body.", fun.name) });
                });

                self.functions.push(FunctionWithoutBody {
                    pointer: fun,
                    decorators: syntax.decorators.clone(),
                    body,
                });
            }
            _ => {
                return Err(LinkError::LinkError { msg: format!("Statement {:?} not valid in a conformance context.", statement) });
            }
        }

        Ok(())
    }

    pub fn finalize(&self, module: &mut Module) -> Result<(), LinkError> {
        let mut function_bindings = HashMap::new();
        let mut unmatched_implementations = self.functions.iter().map(|f| Rc::clone(&f.pointer)).collect_vec();

        for abstract_function in self.binding.trait_.abstract_functions.iter() {
            let expected_interface = Rc::new(map_interface_types(&abstract_function.target.interface, &|type_| type_.replacing_any(&self.binding.generic_to_type)));
            let mut expected_pointer = Rc::new(FunctionPointer {
                pointer_id: Default::default(),
                target: Rc::new(Function { function_id: Default::default(), interface: expected_interface }),
                call_type: abstract_function.call_type.clone(),
                name: abstract_function.name.clone(),
                form: abstract_function.form.clone(),
            });

            let matching_implementations = unmatched_implementations.iter().enumerate()
                .filter(|(i, pointer)| pointer.can_match(&expected_pointer))
                .map(|(i, interface)| i)
                .collect_vec();

            if matching_implementations.len() == 0 {
                return Err(LinkError::LinkError { msg: format!("Function {:?} missing for conformance.", expected_pointer) });
            }
            else if matching_implementations.len() > 1 {
                return Err(LinkError::LinkError { msg: format!("Function {:?} is implemented multiple times.", expected_pointer) });
            }
            else {
                function_bindings.insert(
                    Rc::clone(abstract_function),
                    unmatched_implementations.remove(matching_implementations[0])
                );
            }
        }

        if unmatched_implementations.len() > 0 {
            return Err(LinkError::LinkError { msg: format!("Unrecognized functions for declaration {:?}: {:?}.", self.binding, unmatched_implementations) });
        }

        module.trait_conformance.add_conformance(Rc::clone(&self.binding), function_bindings)
    }
}
