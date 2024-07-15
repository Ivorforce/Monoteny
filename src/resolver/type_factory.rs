use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use itertools::Itertools;

use crate::ast;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::parser::expressions;
use crate::program::functions::FunctionTargetType;
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};
use crate::resolver::scopes;

pub struct TypeFactory<'a> {
    pub runtime: &'a Runtime,
    pub scope: &'a scopes::Scope<'a>,

    pub generics: HashMap<String, Rc<Trait>>,
    pub requirements: HashSet<Rc<TraitBinding>>,
}

// TODO Essentially this is a form of mini interpreter.
//  In the future it might be easier to rewrite it as such.
impl <'a> TypeFactory<'a> {
    pub fn new(scope: &'a scopes::Scope<'a>, runtime: &'a Runtime) -> TypeFactory<'a> {
        TypeFactory {
            scope,
            generics: HashMap::new(),
            requirements: HashSet::new(),
            runtime,
        }
    }

    pub fn resolve_trait(&mut self, name: &str) -> RResult<Rc<Trait>> {
        let reference = self.scope.resolve(FunctionTargetType::Global, &name)?;
        let overload = reference.as_function_overload()?;

        let function = overload.functions.iter().exactly_one()
            .map_err(|_| RuntimeError::error("Function overload cannot be resolved to a type.").to_array())?;
        let trait_ = self.runtime.source.trait_references.get(function)
            .ok_or_else(|| RuntimeError::error(format!("Interpreted types aren't supported yet; please use an explicit type for now.\n{}", name).as_str()).to_array())?;

        return Ok(Rc::clone(trait_))
    }

    fn register_generic(&mut self, name: &str) -> Rc<Trait> {
        let trait_ = Rc::new(Trait::new_flat(name));
        self.generics.insert(name.to_string(), Rc::clone(&trait_));
        trait_
    }

    fn register_requirement(&mut self, requirement: Rc<TraitBinding>) {
        self.requirements.insert(requirement);
    }

    pub fn resolve_type(&mut self, syntax: &ast::Expression, allow_anonymous_generics: bool) -> RResult<Rc<TypeProto>> {
        syntax.no_errors()?;

        let parsed = expressions::parse(syntax, &self.scope.grammar)?;

        let expressions::Value::Identifier(identifier) = &parsed.value else {
            return Err(RuntimeError::error("Interpreted types aren't supported yet; please use an explicit type for now.").in_range(parsed.position).to_array())
        };

        self.resolve_type_by_name(allow_anonymous_generics, &identifier)
            .err_in_range(&parsed.position)
    }

    fn resolve_type_by_name(&mut self, allow_anonymous_generics: bool, type_name: &str) -> RResult<Rc<TypeProto>> {
        let arguments = vec![];

        if let Some(type_) = self.generics.get(type_name) {
            return Ok(TypeProto::unit_struct(type_))
        }

        if !allow_anonymous_generics || !(type_name.starts_with("#") || type_name.starts_with("$")) {
            // No special generic; let's try just resolving it normally.
            let trait_ = self.resolve_trait(type_name)?;
            // Found a trait! Until we actually interpret the expression, this is guaranteed to be unbound.
            return Ok(TypeProto::unit_struct(&trait_));
        }

        let type_ = Rc::new(TypeProto {
            unit: TypeUnit::Struct(self.register_generic(type_name).clone()),
            arguments
        });

        if type_name.starts_with("$") {
            let type_name = match type_name.find("#") {
                None => { String::from(&type_name[1..]) }
                Some(hash_start_index) => { String::from(&type_name[1..hash_start_index]) }
            };

            let requirement_trait = self.resolve_trait(&type_name)?;
            self.register_requirement(Rc::new(TraitBinding {
                generic_to_type: HashMap::from([(Rc::clone(&requirement_trait.generics["Self"]), type_.clone())]),
                trait_: requirement_trait,
            }));
        }

        Ok(type_)
    }
}
