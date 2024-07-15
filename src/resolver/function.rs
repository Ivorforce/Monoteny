use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::ast;
use crate::error::RResult;
use crate::interpreter::runtime::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::ExpressionTree;
use crate::program::functions::FunctionHead;
use crate::program::generics::TypeForest;
use crate::program::global::FunctionImplementation;
use crate::program::traits::{RequirementsAssumption, TraitConformance, TraitConformanceRule};
use crate::resolver::imperative::ImperativeResolver;
use crate::resolver::imperative_builder::ImperativeBuilder;
use crate::resolver::scopes;

pub fn resolve_function_body(head: Rc<FunctionHead>, body: &ast::Expression, scope: &scopes::Scope, runtime: &mut Runtime) -> RResult<Box<FunctionImplementation>> {
    let mut scope = scope.subscope();

    let granted_requirements = scope.trait_conformance.assume_granted(
        head.interface.requirements.iter().cloned()
    );

    add_conformances_to_scope(&mut scope, &granted_requirements)?;

    let mut builder = ImperativeBuilder {
        runtime,
        types: Box::new(TypeForest::new()),
        expression_tree: Box::new(ExpressionTree::new(Uuid::new_v4())),
        locals_names: Default::default(),
    };

    // Register parameters as variables.
    let mut parameter_variables = vec![];
    for parameter in head.interface.parameters.clone() {
        let parameter_variable = ObjectReference::new_immutable(parameter.type_.clone());
        _ = builder.register_local(&parameter.internal_name, Rc::clone(&parameter_variable), &mut scope)?;
        parameter_variables.push(parameter_variable);
    }

    let mut resolver = ImperativeResolver {
        return_type: Rc::clone(&head.interface.return_type),
        builder,
        ambiguities: vec![],
    };

    let head_expression = resolver.resolve_expression(body, &scope)?;
    resolver.builder.types.bind(head_expression, &head.interface.return_type)?;
    resolver.builder.expression_tree.root = head_expression;  // TODO This is kinda dumb; but we can't write into an existing head expression
    resolver.resolve_all_ambiguities()?;

    Ok(Box::new(FunctionImplementation {
        head,
        requirements_assumption: Box::new(RequirementsAssumption { conformance: HashMap::from_iter(granted_requirements.into_iter().map(|c| (Rc::clone(&c.binding), c))) }),
        expression_tree: resolver.builder.expression_tree,
        type_forest: resolver.builder.types,
        parameter_locals: parameter_variables,
        locals_names: resolver.builder.locals_names,
    }))
}

fn add_conformances_to_scope(scope: &mut scopes::Scope, granted_requirements: &Vec<Rc<TraitConformance>>) -> RResult<()> {
    // TODO Register generic types as variables so they can be referenced in the function

    // Let our scope know that our parameter types (all of type any!) conform to the requirements
    for conformance in granted_requirements.iter() {
        scope.trait_conformance.add_conformance_rule(TraitConformanceRule::direct(
            Rc::clone(conformance),
        ));
    };

    // Add abstract function mocks to our scope to be callable.
    for conformance in granted_requirements.iter() {
        for (abstract_function, function) in conformance.function_mapping.iter() {
            // TODO Do we need to keep track of the object reference created by this trait conformance?
            //  For the record, it SHOULD be created - an abstract function reference can still be passed around,
            //  assigned and maybe called later.
            scope.overload_function(
                function,
                abstract_function.declared_representation.clone(),
            )?;
        }
    }

    Ok(())
}
