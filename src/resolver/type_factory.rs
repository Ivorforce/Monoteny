use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::parser::expressions;
use crate::program::functions::FunctionTargetType;
use crate::program::traits::{Nominal, NominalKind, TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};
use crate::resolver::scopes;
use itertools::Itertools;

pub struct TypeFactory<'a> {
    pub scope: &'a scopes::Scope<'a>,

    pub generics: HashMap<String, Rc<Nominal>>,
    pub requirements: HashSet<Rc<TraitBinding>>,
}

// TODO Essentially this is a form of mini interpreter.
//  In the future it might be easier to rewrite it as such.
impl <'a> TypeFactory<'a> {
    pub fn new(scope: &'a scopes::Scope<'a>) -> TypeFactory<'a> {
        TypeFactory {
            scope,
            generics: HashMap::new(),
            requirements: HashSet::new(),
        }
    }

    pub fn resolve_nominal(&mut self, name: &str, runtime: &mut Runtime) -> RResult<Rc<Nominal>> {
        let reference = self.scope.resolve(FunctionTargetType::Global, &name)?;
        let overload = reference.as_function_overload()?;

        let function = overload.functions.iter().exactly_one()
            .map_err(|_| RuntimeError::error("Function overload cannot be resolved to a type.").to_array())?;
        let trait_ = runtime.source.nominal_references.get(function)
            .ok_or_else(|| RuntimeError::error(format!("Interpreted types aren't supported yet; please use an explicit type for now.\n{}", name).as_str()).to_array())?;

        return Ok(Rc::clone(trait_))
    }

    fn register_generic(&mut self, name: &str) -> Rc<Nominal> {
        let trait_ = Rc::new(Nominal::new_flat(name));
        self.generics.insert(name.to_string(), Rc::clone(&trait_));
        trait_
    }

    fn register_requirement(&mut self, requirement: Rc<TraitBinding>) {
        self.requirements.insert(requirement);
    }

    pub fn resolve_type(&mut self, syntax: &ast::Expression, allow_anonymous_generics: bool, runtime: &mut Runtime) -> RResult<Rc<TypeProto>> {
        syntax.no_errors()?;

        let parsed = expressions::parse(syntax, &self.scope.grammar)?;

        let expressions::Value::Identifier(identifier) = &parsed.value else {
            return Err(RuntimeError::error("Interpreted types aren't supported yet; please use an explicit type for now.").in_range(parsed.position).to_array())
        };

        // let (expression, _) = parse_expression(identifier)?;
        // // TODO We don't actually want to merge into Metatype<String>, we want an instance of
        // let result = runtime.evaluate_anonymous_expression(
        //     &expression,
        //     FunctionInterface::new_provider(
        //         &TypeProto::one_arg(&runtime.Metatype, TypeProto::unit_struct(&runtime.traits.as_ref().unwrap().String)),
        //         vec![]
        //     ),
        // )?;
        //
        // unsafe {
        //     let uuid = *(result.ptr as *mut Uuid);
        //     return Ok(TypeProto::unit_struct(&runtime.source.nominal_heads[&uuid]));
        // }

        self.resolve_type_by_name(allow_anonymous_generics, &identifier, runtime)
            .err_in_range(&parsed.position)
    }

    fn resolve_type_by_name(&mut self, allow_anonymous_generics: bool, type_name: &str, runtime: &mut Runtime) -> RResult<Rc<TypeProto>> {
        // A name we've already invented a generic for (shared by name within this function).
        if let Some(type_) = self.generics.get(type_name) {
            return Ok(TypeProto::unit_struct(type_))
        }

        // `#` / `#A` is an unconstrained anonymous generic with no backing trait.
        if type_name.starts_with("#") {
            if !allow_anonymous_generics {
                return Err(RuntimeError::error(format!("Anonymous generic '{}' is not allowed here.", type_name).as_str()).to_array());
            }
            return Ok(TypeProto::unit_struct(&self.register_generic(type_name)));
        }

        // A `Trait#label` discriminator names a distinct generic; the trait is the part before '#'.
        let (base_name, has_discriminator) = match type_name.find("#") {
            None => (type_name, false),
            Some(hash_start_index) => (&type_name[..hash_start_index], true),
        };

        let nominal = self.resolve_nominal(base_name, runtime)?;

        match nominal.kind {
            // A concrete type (struct or builtin primitive) always resolves to itself.
            NominalKind::Struct => {
                if has_discriminator {
                    return Err(RuntimeError::error(format!("Concrete type '{}' cannot take a generic discriminator ('#').", base_name).as_str()).to_array());
                }
                Ok(TypeProto::unit_struct(&nominal))
            }
            // A generic placeholder (e.g. `Self`) refers to itself.
            NominalKind::Generic => Ok(TypeProto::unit_struct(&nominal)),
            // A bare trait name invents one generic conforming to it (shared by name within
            // this function); `Trait#A` / `Trait#B` create distinct generics. The exception
            // is a position that disallows generics (e.g. the `is` side of a conformance),
            // where the trait refers to itself.
            NominalKind::Trait => {
                if !allow_anonymous_generics {
                    return Ok(TypeProto::unit_struct(&nominal));
                }
                let type_ = Rc::new(TypeProto {
                    unit: TypeUnit::Struct(self.register_generic(type_name)),
                    arguments: vec![],
                });
                self.register_requirement(Rc::new(TraitBinding {
                    generic_to_type: HashMap::from([(Rc::clone(&nominal.generics["Self"]), type_.clone())]),
                    trait_: nominal,
                }));
                Ok(type_)
            }
        }
    }
}
