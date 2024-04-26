use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use display_with_options::with_options;

use itertools::Itertools;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::resolver::interface::resolve_function_interface;
use crate::resolver::scopes;
use crate::parser::ast;
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::FunctionHead;
use crate::program::traits::{Trait, TraitBinding, TraitConformance};
use crate::refactor::monomorphize::map_interface_types;

pub struct UnresolvedFunctionImplementation<'a> {
    pub function: Rc<FunctionHead>,
    pub representation: FunctionRepresentation,
    pub decorators: Vec<String>,
    pub body: &'a Option<ast::Expression>,
}

pub struct ConformanceResolver<'a, 'b> {
    pub runtime: &'b Runtime,
    pub functions: Vec<UnresolvedFunctionImplementation<'a>>,
}

impl <'a, 'b> ConformanceResolver<'a, 'b> {
    pub fn resolve_statement(&mut self, statement: &'a ast::Statement, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashMap<String, Rc<Trait>>, scope: &scopes::Scope) -> RResult<()> {
        match statement {
            ast::Statement::FunctionDeclaration(syntax) => {
                // TODO For simplicity's sake, we should match the generics IDs of all conformances
                //  to the ID of the parent abstract function. That way, we can avoid another
                //  generic to generic mapping later.
                let (function, representation) = resolve_function_interface(&syntax.interface, &scope, None, &self.runtime, requirements, generics)?;

                self.functions.push(UnresolvedFunctionImplementation {
                    function,
                    representation,
                    body: &syntax.body,
                    decorators: vec![],
                });
            }
            ast::Statement::Expression(e) => {
                return Err(
                    RuntimeError::error(format!("Expression {} not valid in a conformance context.", statement).as_str()).to_array()
                );
            }
            ast::Statement::Error(err) => Err(err.clone().to_array())?,
            _ => {
                return Err(
                    RuntimeError::error(format!("Statement {} not valid in a conformance context.", statement).as_str()).to_array()
                );
            }
        }

        Ok(())
    }

    pub fn finalize_conformance(&self, binding: Rc<TraitBinding>, conformance_requirements: &HashSet<Rc<TraitBinding>>, conformance_generics: &HashMap<String, Rc<Trait>>) -> RResult<Rc<TraitConformance>> {
        let mut function_bindings = HashMap::new();
        let mut unmatched_implementations = self.functions.iter().collect_vec();

        for (abstract_function, abstract_representation) in binding.trait_.abstract_functions.iter() {
            let mut expected_interface = map_interface_types(&abstract_function.interface, &binding.generic_to_type);
            expected_interface.requirements.extend(conformance_requirements.clone());
            expected_interface.generics.extend(conformance_generics.clone());

            let matching_implementations = unmatched_implementations.iter().enumerate()
                .filter(|(i, imp)| &imp.representation == abstract_representation && imp.function.interface.as_ref() == &expected_interface)
                .map(|(i, interface)| i)
                .collect_vec();

            if matching_implementations.len() == 0 {
                return Err(
                    RuntimeError::error(format!("Function {:?} missing for conformance.", with_options(&expected_interface, abstract_representation)).as_str()).to_array()
                );
            }
            else if matching_implementations.len() > 1 {
                return Err(
                    RuntimeError::error(format!("Function {:?} is implemented multiple times.", with_options(&expected_interface, abstract_representation)).as_str()).to_array()
                );
            }
            else {
                function_bindings.insert(
                    Rc::clone(abstract_function),
                    Rc::clone(&unmatched_implementations.remove(matching_implementations[0]).function)
                );
            }
        }

        if unmatched_implementations.len() > 0 {
            return Err(
                RuntimeError::error(format!("Unrecognized functions for declaration {:?}: {:?}.", binding, unmatched_implementations).as_str()).to_array()
            );
        }

        Ok(TraitConformance::new(Rc::clone(&binding), function_bindings.clone()))
    }
}

impl<'a> Debug for UnresolvedFunctionImplementation<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", with_options(self.function.as_ref(), &self.representation))
    }
}
