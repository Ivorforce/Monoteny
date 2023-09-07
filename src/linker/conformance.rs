use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::generic_unfolding::map_interface_types;
use crate::linker::{LinkError, scopes};
use crate::linker::global::UnlinkedFunctionImplementation;
use crate::linker::interface::link_function_pointer;
use crate::parser::ast;
use crate::program::builtins::Builtins;
use crate::program::functions::{FunctionHead, FunctionPointer};
use crate::program::module::Module;
use crate::program::traits::{TraitBinding, TraitConformance};

pub struct ConformanceLinker<'a> {
    pub builtins: &'a Builtins,

    pub functions: Vec<UnlinkedFunctionImplementation<'a>>,
}

impl <'a> ConformanceLinker<'a> {
    pub fn link_statement(&mut self, statement: &'a ast::GlobalStatement, requirements: &HashSet<Rc<TraitBinding>>, scope: &scopes::Scope) -> Result<(), LinkError> {
        match statement {
            ast::GlobalStatement::FunctionDeclaration(syntax) => {
                // TODO For simplicity's sake, we should match the generics IDs of all conformances
                //  to the ID of the parent abstract function. That way, we can avoid another
                //  generic to generic mapping later.
                let fun = link_function_pointer(&syntax, &scope, requirements)?;
                guard!(let Some(body) = &syntax.body else {
                    return Err(LinkError::LinkError { msg: format!("Function {} needs a body.", fun.name) });
                });

                self.functions.push(UnlinkedFunctionImplementation {
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

    pub fn finalize(&self, binding: Rc<TraitBinding>, requirements: HashSet<Rc<TraitBinding>>, module: &mut Module, scope: &mut scopes::Scope) -> Result<(), LinkError> {
        let mut function_bindings = HashMap::new();
        let mut unmatched_implementations = self.functions.iter().map(|f| Rc::clone(&f.pointer)).collect_vec();

        for abstract_function in binding.trait_.abstract_functions.values() {
            let expected_interface = Rc::new(map_interface_types(&abstract_function.target.interface, &|type_| type_.replacing_generics(&binding.generic_to_type)));
            let expected_pointer = Rc::new(FunctionPointer {
                target: Rc::new(FunctionHead {
                    function_id: Uuid::new_v4(),
                    function_type: abstract_function.target.function_type.clone(),
                    interface: expected_interface,
                }),
                name: abstract_function.name.clone(),
                form: abstract_function.form.clone(),
            });

            let matching_implementations = unmatched_implementations.iter().enumerate()
                .filter(|(i, ptr)| ptr.can_match_strict(&expected_pointer))
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
                    Rc::clone(&abstract_function.target),
                    Rc::clone(&unmatched_implementations.remove(matching_implementations[0]).target)
                );
            }
        }

        if unmatched_implementations.len() > 0 {
            return Err(LinkError::LinkError { msg: format!("Unrecognized functions for declaration {:?}: {:?}.", binding, unmatched_implementations) });
        }

        let conformance = TraitConformance::new(Rc::clone(&binding), function_bindings.clone());
        module.trait_conformance.add_conformance_rule(requirements.clone(), Rc::clone(&conformance));
        scope.traits.add_conformance_rule(requirements, conformance);
        Ok(())
    }
}