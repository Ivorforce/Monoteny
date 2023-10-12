use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::monomorphize::map_interface_types;
use crate::linker::scopes;
use crate::linker::interface::link_function_interface;
use crate::parser::ast;
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::FunctionHead;
use crate::program::traits::{TraitBinding, TraitConformance};
use crate::util::fmt::fmta;

pub struct UnlinkedFunctionImplementation<'a> {
    pub function: Rc<FunctionHead>,
    pub representation: FunctionRepresentation,
    pub decorators: Vec<String>,
    pub body: &'a Option<ast::Expression>,
}

pub struct ConformanceLinker<'a, 'b> {
    pub runtime: &'b Runtime,
    pub functions: Vec<UnlinkedFunctionImplementation<'a>>,
}

impl <'a, 'b> ConformanceLinker<'a, 'b> {
    pub fn link_statement(&mut self, statement: &'a ast::Statement, requirements: &HashSet<Rc<TraitBinding>>, scope: &scopes::Scope) -> RResult<()> {
        match statement {
            ast::Statement::FunctionDeclaration(syntax) => {
                // TODO For simplicity's sake, we should match the generics IDs of all conformances
                //  to the ID of the parent abstract function. That way, we can avoid another
                //  generic to generic mapping later.
                let (function, representation) = link_function_interface(&syntax.interface, &scope, None, &self.runtime, requirements)?;

                self.functions.push(UnlinkedFunctionImplementation {
                    function,
                    representation,
                    body: &syntax.body,
                    decorators: vec![],
                });
            }
            _ => {
                return Err(RuntimeError::new(format!("Statement {} not valid in a conformance context.", statement)));
            }
        }

        Ok(())
    }

    pub fn finalize_conformance(&self, binding: Rc<TraitBinding>, conformance_requirements: &HashSet<Rc<TraitBinding>>) -> RResult<Rc<TraitConformance>> {
        let mut function_bindings = HashMap::new();
        let mut unmatched_implementations = self.functions.iter().collect_vec();

        for (abstract_function, abstract_representation) in binding.trait_.abstract_functions.iter() {
            let mut expected_interface = map_interface_types(&abstract_function.interface, &binding.generic_to_type);
            expected_interface.requirements.extend(conformance_requirements.clone());

            let matching_implementations = unmatched_implementations.iter().enumerate()
                .filter(|(i, imp)| &imp.representation == abstract_representation && imp.function.interface.as_ref() == &expected_interface)
                .map(|(i, interface)| i)
                .collect_vec();

            if matching_implementations.len() == 0 {
                return Err(RuntimeError::new(format!("Function {:?} missing for conformance.", fmta(|f| expected_interface.format(f, abstract_representation)))));
            }
            else if matching_implementations.len() > 1 {
                return Err(RuntimeError::new(format!("Function {:?} is implemented multiple times.", fmta(|f| expected_interface.format(f, abstract_representation)))));
            }
            else {
                function_bindings.insert(
                    Rc::clone(abstract_function),
                    Rc::clone(&unmatched_implementations.remove(matching_implementations[0]).function)
                );
            }
        }

        if unmatched_implementations.len() > 0 {
            return Err(RuntimeError::new(format!("Unrecognized functions for declaration {:?}: {:?}.", binding, unmatched_implementations)));
        }

        Ok(TraitConformance::new(Rc::clone(&binding), function_bindings.clone()))
    }
}

impl<'a> Debug for UnlinkedFunctionImplementation<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.function.format(f, &self.representation)
    }
}
