use std::collections::HashMap;
use std::rc::Rc;

use itertools::Itertools;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation, ExpressionTree};
use crate::program::functions::{FunctionBinding, FunctionOverload, FunctionTargetType};
use crate::program::generics::TypeForest;
use crate::program::types::TypeProto;
use crate::resolver::scopes;

/// Note: This object should not know about the AST.
pub struct ImperativeBuilder<'a> {
    pub runtime: &'a Runtime,
    pub types: Box<TypeForest>,
    pub expression_tree: Box<ExpressionTree>,
    pub locals_names: HashMap<Rc<ObjectReference>, String>,
}

impl<'a> ImperativeBuilder<'a> {
    pub fn make_expression(&mut self, arguments: Vec<ExpressionID>) -> ExpressionID {
        let id = ExpressionID::new_v4();

        self.types.register(id);
        for argument in arguments.iter() {
            self.expression_tree.parents.insert(*argument, id);
        }
        self.expression_tree.children.insert(id, arguments);

        id
    }

    pub fn make_operation_expression(&mut self, arguments: Vec<ExpressionID>, operation: ExpressionOperation) -> ExpressionID {
        let id = self.make_expression(arguments);
        self.expression_tree.values.insert(id.clone(), operation);
        id
    }

    pub fn make_full_expression(&mut self, arguments: Vec<ExpressionID>, return_type: &TypeProto, operation: ExpressionOperation) -> RResult<ExpressionID> {
        let id = self.make_expression(arguments);

        self.expression_tree.values.insert(id.clone(), operation);

        self.types.bind(id, &return_type)
            .map(|_| id)
    }

    pub fn register_local(&mut self, identifier: &str, reference: Rc<ObjectReference>, scope: &mut scopes::Scope) -> RResult<()> {
        self.locals_names.insert(Rc::clone(&reference), identifier.to_string());
        scope.override_reference(FunctionTargetType::Global, scopes::Reference::Local(reference), identifier)
    }

    pub fn add_string_primitive(&mut self, value: &str) -> RResult<ExpressionID> {
        self.make_full_expression(
            vec![],
            &TypeProto::unit_struct(&self.runtime.traits.as_ref().unwrap().String),
            ExpressionOperation::StringLiteral(value.to_string())
        )
    }

    pub fn add_function_reference(&mut self, overload: &Rc<FunctionOverload>) -> RResult<ExpressionID> {
        match overload.functions.iter().exactly_one() {
            Ok(function) => {
                let getter = &self.runtime.source.fn_getters[function];
                let expression_id = self.make_full_expression(
                    vec![],
                    &getter.interface.return_type,
                    // Call the getter of the function 'object' instead of the function itself.
                    ExpressionOperation::FunctionCall(FunctionBinding::pure(Rc::clone(getter)))
                )?;

                Ok(expression_id)
            }
            _ => return Err(
                RuntimeError::error("References to overloaded functions are not yet supported (need syntax to distinguish which to choose).").to_array()
            )?,
        }
    }
}
