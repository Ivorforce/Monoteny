use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::linker::grammar::{Grammar, Pattern, PrecedenceGroup};
use crate::program::allocation::Reference;
use crate::program::function_object::{FunctionForm, FunctionOverload, FunctionRepresentation};
use crate::program::functions::FunctionHead;
use crate::program::traits::TraitGraph;
use crate::program::module::Module;

// Note: While a single pool cannot own overloaded variables, multiple same-level pools (-> from imports) can.
// When we have imports, this should be ignored until referenced, to avoid unnecessary import complications.
// For these cases, we could store an AMBIGUOUS value inside our pool, crashing when accessed?
type RefPool = HashMap<String, Reference>;

#[derive(Copy, Clone, PartialEq)]
pub enum Environment {
    Global,
    Member
}

pub struct Scope<'a> {
    pub parent: Option<&'a Scope<'a>>,

    pub trait_conformance: TraitGraph,
    pub grammar: Grammar,

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

    pub fn references_mut(&mut self, environment: Environment) -> &mut RefPool {
        match environment {
            Environment::Global => &mut self.global,
            Environment::Member => &mut self.member
        }
    }

    pub fn references(&self, environment: Environment) -> &RefPool {
        match environment {
            Environment::Global => &self.global,
            Environment::Member => &self.member
        }
    }

    pub fn import(&mut self, module: &Module, runtime: &Runtime) -> RResult<()> {
        // This wipes any existing patterns, but I think that's what we want.
        if let Some(precedence) = &module.precedence_order {
            self.grammar.set_precedence_order(precedence.clone());
        }

        for pattern in module.patterns.iter() {
            self.add_pattern(Rc::clone(pattern))?;
        }

        for function in module.exposed_functions.iter() {
            let representation = &runtime.source.fn_representations[function];
            self.overload_function(function, representation.clone())?;
        }

        self.trait_conformance.add_graph(&module.trait_conformance);

        Ok(())
    }

    pub fn overload_function(&mut self, fun: &Rc<FunctionHead>, representation: FunctionRepresentation) -> RResult<()> {
        let name = &representation.name;
        let environment = Environment::from_form(&representation.form);

        let mut refs = self.references_mut(environment);

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
            if let Some(Some(Reference::FunctionOverload(overload))) = self.parent.map(|x| x.resolve(environment, name).ok()) {
                let overload = Reference::FunctionOverload(overload.adding_function(fun)?);

                let mut refs = self.references_mut(environment);
                refs.insert(representation.name.clone(), overload);
            }

            let mut refs = self.references_mut(environment);

            let overload = Reference::FunctionOverload(FunctionOverload::from(fun, representation.clone()));

            refs.insert(representation.name.clone(), overload);
        }

        Ok(())
    }

    pub fn insert_singleton(&mut self, environment: Environment, reference: Reference, name: &str) {
        let mut refs = self.references_mut(environment);

        if let Some(other) = refs.insert(name.to_string(), reference) {
            panic!("Multiple references with the same name: {}", name);
        }
    }

    pub fn insert_keyword(&mut self, keyword: &str) {
        let reference = Reference::Keyword(keyword.to_string());
        let mut refs = self.references_mut(Environment::Global);

        if let Some(other) = refs.insert(keyword.to_string(), reference) {
            if Reference::Keyword(keyword.to_string()) != other {
                panic!("Multiple references with the same name: {}", keyword);
            }
        }
    }

    pub fn add_pattern(&mut self, pattern: Rc<Pattern>) -> RResult<()> {
        for keyword in self.grammar.add_pattern(pattern)? {
            self.insert_keyword(&keyword);
        }
        Ok(())
    }

    pub fn override_reference(&mut self, environment: Environment, reference: Reference, name: &str) {
        let mut refs = self.references_mut(environment);

        refs.insert(name.to_string(), reference);
    }

    pub fn contains(&mut self, environment: Environment, name: &str) -> bool {
        self.references(environment).contains_key(name)
    }
}

impl <'a> Scope<'a> {
    pub fn resolve(&'a self, environment: Environment, name: &str) -> RResult<&'a Reference> {
        let mut scope = self;
        loop {
            if let Some(reference) = scope.references(environment).get(name) {
                return Ok(reference)
            }

            if let Some(parent) = scope.parent {
                scope = parent;
            }
            else {
                // take that rust, i steal ur phrasings
                let env_part = match environment {
                    Environment::Global => "",
                    Environment::Member => "."
                };

                return Err(RuntimeError::new(format!("Cannot find '{}{}' in this scope", env_part, name)))
            }
        }
    }

    pub fn resolve_precedence_group(&self, name: &str) -> RResult<Rc<PrecedenceGroup>> {
        for group in self.grammar.groups_and_keywords.keys() {
            if &group.name == name {
                return Ok(Rc::clone(group))
            }
        }

        return Err(RuntimeError::new(format!("Precedence group could not be resolved: {}", name)))
    }
}

impl Environment {
    pub fn from_form(form: &FunctionForm) -> Environment {
        match form {
            FunctionForm::MemberFunction => Environment::Member,
            FunctionForm::MemberImplicit => Environment::Member,
            FunctionForm::GlobalFunction => Environment::Global,
            FunctionForm::GlobalImplicit => Environment::Global,
        }
    }
}
