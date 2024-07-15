use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::parser::grammar::{Grammar, PrecedenceGroup};
use crate::program::allocation::ObjectReference;
use crate::program::functions::{FunctionHead, FunctionOverload, FunctionRepresentation, FunctionTargetType};
use crate::program::module::Module;
use crate::program::traits::TraitGraph;

// Note: While a single pool cannot own overloaded variables, multiple same-level pools (-> from imports) can.
// When we have imports, this should be ignored until referenced, to avoid unnecessary import complications.
// For these cases, we could store an AMBIGUOUS value inside our pool, crashing when accessed?
type RefPool = HashMap<String, Reference>;

pub struct Scope<'a> {
    pub parent: Option<&'a Scope<'a>>,

    pub trait_conformance: TraitGraph,
    pub grammar: Grammar<Rc<FunctionHead>>,

    pub global: RefPool,
    pub member: RefPool,
}

impl <'a> Scope<'a> {
    pub fn new() -> Scope<'a> {
        Scope {
            parent: None,

            trait_conformance: TraitGraph::new(),
            grammar: Grammar::new(),

            global: HashMap::new(),
            member: HashMap::new(),
        }
    }

    // TODO When importing, make a new scope out of combined imports, and set that as parent
    //  of our current scope.

    pub fn subscope(&'a self) -> Scope<'a> {
        Scope {
            parent: Some(self),

            trait_conformance: self.trait_conformance.clone(),
            grammar: self.grammar.clone(),

            global: HashMap::new(),
            member: HashMap::new(),
        }
    }

    pub fn references_mut(&mut self, environment: FunctionTargetType) -> &mut RefPool {
        match environment {
            FunctionTargetType::Global => &mut self.global,
            FunctionTargetType::Member => &mut self.member
        }
    }

    pub fn references(&self, environment: FunctionTargetType) -> &RefPool {
        match environment {
            FunctionTargetType::Global => &self.global,
            FunctionTargetType::Member => &self.member
        }
    }

    pub fn not_a_keyword(&self, keyword: &str) -> RResult<()> {
        return if self.grammar.keywords.contains(keyword) {
            Err(RuntimeError::error("Cannot shadow keyword.").to_array())
        } else {
            Ok(())
        }
    }

    pub fn import(&mut self, module: &Module, runtime: &Runtime) -> RResult<()> {
        // This wipes any existing patterns, but I think that's what we want.
        if let Some(precedence) = &module.precedence_order {
            self.grammar.set_precedence_order(precedence.clone());
        }

        for pattern in module.patterns.iter() {
            self.grammar.add_pattern(Rc::clone(pattern))?;
        }

        for function in module.exposed_functions.iter() {
            self.overload_function(function, function.declared_representation.clone())?;
        }

        self.trait_conformance.add_graph(&module.trait_conformance);

        Ok(())
    }

    pub fn overload_function(&mut self, fun: &Rc<FunctionHead>, representation: FunctionRepresentation) -> RResult<()> {
        let name = &representation.name;
        self.not_a_keyword(name)?;

        let mut refs = self.references_mut(representation.target_type);

        // Remove the current FunctionOverload reference and replace with a reference containing also our new overload.
        // This may seem weird at first but it kinda makes sense - if someone queries the scope, gets a reference,
        // and then the scope is modified, the previous caller still expects their reference to not change.
        if let Some(existing) = refs.remove(name) {
            if let Reference::FunctionOverload(overload) = existing {
                let overload = Reference::FunctionOverload(overload.adding_function(fun)?);

                refs.insert(representation.name.clone(), overload);
            }
            else {
                panic!("Cannot overload with function '{}' if a reference exists in the same scope under the same name.", name);
            }
        }
        else {
            // Copy the parent's function overload into us and then add the function to the overload
            if let Some(Some(Reference::FunctionOverload(overload))) = self.parent.map(|x| x.resolve(representation.target_type, name).ok()) {
                let overload = Reference::FunctionOverload(overload.adding_function(fun)?);

                let mut refs = self.references_mut(representation.target_type);
                refs.insert(representation.name.clone(), overload);
            }

            let mut refs = self.references_mut(representation.target_type);

            let overload = Reference::FunctionOverload(FunctionOverload::from(fun, representation.clone()));

            refs.insert(representation.name.clone(), overload);
        }

        Ok(())
    }

    pub fn insert_singleton(&mut self, target_type: FunctionTargetType, reference: Reference, name: &str) -> RResult<()> {
        self.not_a_keyword(name)?;
        let mut refs = self.references_mut(target_type);

        if let Some(other) = refs.insert(name.to_string(), reference) {
            return Err(RuntimeError::error(format!("Multiple references with this name: {}", name).as_str()).to_array());
        }
        Ok(())
    }

    pub fn override_reference(&mut self, target_type: FunctionTargetType, reference: Reference, name: &str) -> RResult<()> {
        self.not_a_keyword(name)?;
        let mut refs = self.references_mut(target_type);

        refs.insert(name.to_string(), reference);
        Ok(())
    }

    pub fn contains(&mut self, target_type: FunctionTargetType, name: &str) -> bool {
        self.references(target_type).contains_key(name)
    }
}

impl <'a> Scope<'a> {
    pub fn resolve(&'a self, target_type: FunctionTargetType, name: &str) -> RResult<&'a Reference> {
        let mut scope = self;
        loop {
            if let Some(reference) = scope.references(target_type).get(name) {
                return Ok(reference)
            }

            if let Some(parent) = scope.parent {
                scope = parent;
            }
            else {
                // take that rust, i steal ur phrasings
                let env_part = match target_type {
                    FunctionTargetType::Global => "",
                    FunctionTargetType::Member => "."
                };

                return Err(RuntimeError::error(format!("Cannot find '{}{}' in this scope", env_part, name).as_str()).to_array())
            }
        }
    }

    pub fn resolve_precedence_group(&self, name: &str) -> RResult<Rc<PrecedenceGroup>> {
        for group in self.grammar.groups_and_keywords.keys() {
            if &group.name == name {
                return Ok(Rc::clone(group))
            }
        }

        return Err(
            RuntimeError::error(format!("Precedence group could not be resolved: {}", name).as_str()).to_array()
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Reference {
    // TODO WE can probably get rid of locals if we replace them by getters and setters.
    Local(Rc<ObjectReference>),
    // This COULD be an object, but only if it 'inherits' the callable interfaces
    //  from ALL included overloads. Overall, this is probably too confusing and thus not worth
    //  the effort. Rather, as in other languages, we should expect the user to resolve the overload
    //  - either immediately, or by context (e.g. `(should_add ? add : sub)(1, 2)`).
    FunctionOverload(Rc<FunctionOverload>),
}

impl Reference {
    pub fn as_local(&self, require_mutable: bool) -> RResult<&Rc<ObjectReference>> {
        let Reference::Local(obj_ref) = self else {
            return Err(RuntimeError::error("Reference is not a local.").to_array());
        };

        Ok(&obj_ref)
    }

    pub fn as_function_overload(&self) -> RResult<Rc<FunctionOverload>> {
        match self {
            Reference::FunctionOverload(overload) => Ok(Rc::clone(overload)),
            _ => Err(RuntimeError::error("Reference is not a function.").to_array())
        }
    }
}

impl Debug for Reference {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Reference::Local(t) => write!(fmt, "{:?}", t.type_),
            Reference::FunctionOverload(f) => write!(fmt, "{}", &f.representation.name),
        }
    }
}
