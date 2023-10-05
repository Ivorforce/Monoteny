use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::linker::precedence::PrecedenceGroup;
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::functions::{FunctionForm, FunctionOverload, FunctionPointer};
use crate::program::traits::TraitGraph;
use crate::program::module::Module;
use crate::program::types::{Pattern, PatternPart};

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

    pub traits: TraitGraph,

    pub patterns: HashSet<Rc<Pattern>>,

    /// Contents: For each precedence groups, matched keywords and how they map to which function names
    pub precedence_groups: Vec<(Rc<PrecedenceGroup>, HashMap<String, String>)>,

    pub global: RefPool,
    pub member: RefPool,
}

impl <'a> Scope<'a> {
    pub fn new() -> Scope<'a> {
        Scope {
            parent: None,

            traits: TraitGraph::new(),
            precedence_groups: vec![],

            patterns: HashSet::new(),

            global: HashMap::new(),
            member: HashMap::new(),
        }
    }

    // TODO When importing, make a new scope out of combined imports, and set that as parent
    //  of our current scope.

    pub fn subscope(&'a self) -> Scope<'a> {
        Scope {
            parent: Some(self),

            traits: self.traits.clone(),
            precedence_groups: self.precedence_groups.clone(),

            patterns: self.patterns.clone(),

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

    pub fn import(&mut self, module: &Module) -> RResult<()> {
        for pattern in module.patterns.iter() {
            self.add_pattern(Rc::clone(pattern))?;
        }

        for (trait_, object_ref) in module.traits.iter() {
            self.insert_singleton(
                Environment::Global,
                Reference::Object(Rc::clone(object_ref)),
                &trait_.name.clone()
            );
        }

        self.traits.add_graph(&module.trait_conformance);

        for (function, object_ref) in module.fn_references.iter() {
            self.overload_function(module.fn_pointers.get(function).unwrap(), object_ref)?;
        }

        Ok(())
    }

    pub fn overload_function(&mut self, fun: &Rc<FunctionPointer>, object_ref: &Rc<ObjectReference>) -> RResult<()> {
        let name = &fun.name;
        let environment = Environment::from_form(&fun.form);

        let mut refs = self.references_mut(environment);

        // Remove the current FunctionOverload reference and replace with a reference containing also our new overload.
        // This may seem weird at first but it kinda makes sense - if someone queries the scope, gets a reference,
        // and then the scope is modified, the previous caller still expects their reference to not change.
        if let Some(existing) = refs.remove(name) {
            if let Reference::FunctionOverload(overload) = existing {
                let overload = Reference::FunctionOverload(overload.adding_function(fun, object_ref)?);

                refs.insert(fun.name.clone(), overload);
            }
            else {
                panic!("Cannot overload with function '{}' if a reference exists in the same scope under the same name.", name);
            }
        }
        else {
            // Copy the parent's function overload into us and then add the function to the overload
            if let Some(Some(Reference::FunctionOverload(overload))) = self.parent.map(|x| x.resolve(environment, name).ok()) {
                let overload = Reference::FunctionOverload(overload.adding_function(fun, object_ref)?);

                let mut refs = self.references_mut(environment);
                refs.insert(fun.name.clone(), overload);
            }

            let mut refs = self.references_mut(environment);

            let overload = Reference::FunctionOverload(FunctionOverload::from(fun, object_ref));

            refs.insert(fun.name.clone(), overload);
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

    pub fn override_reference(&mut self, environment: Environment, reference: Reference, name: &str) {
        let mut refs = self.references_mut(environment);

        refs.insert(name.to_string(), reference);
    }

    pub fn contains(&mut self, environment: Environment, name: &str) -> bool {
        self.references(environment).contains_key(name)
    }

    pub fn add_pattern(&mut self, pattern: Rc<Pattern>) -> RResult<()> {
        for (precedence_group, keyword_map) in self.precedence_groups.iter_mut() {
            if precedence_group != &pattern.precedence_group {
                continue;
            }

            match &pattern.parts.iter().map(|x| x.as_ref()).collect_vec()[..] {
                [_] => return Err(RuntimeError::new(format!("Pattern is too short: {}.", pattern.alias))),
                [
                    PatternPart::Keyword(keyword),
                    PatternPart::Parameter { .. },
                ] => {
                    assert_eq!(precedence_group.name, "LeftUnaryPrecedence");
                    keyword_map.insert(keyword.clone(), pattern.alias.clone());
                    self.insert_keyword(keyword);
                },
                [
                    PatternPart::Parameter { .. },
                    PatternPart::Keyword(keyword),
                ] => {
                    assert_eq!(precedence_group.name, "RightUnaryPrecedence");
                    keyword_map.insert(keyword.clone(), pattern.alias.clone());
                    self.insert_keyword(keyword);
                },
                [
                    PatternPart::Parameter { .. },
                    PatternPart::Keyword(keyword),
                    PatternPart::Parameter { .. },
                ] => {
                    assert_ne!(precedence_group.name, "LeftUnaryPrecedence");
                    assert_ne!(precedence_group.name, "RightUnaryPrecedence");
                    keyword_map.insert(keyword.clone(), pattern.alias.clone());
                    self.insert_keyword(keyword);
                }
                _ => return Err(RuntimeError::new(String::from("This pattern form is not supported; try using unary or binary patterns."))),
            };

            self.patterns.insert(pattern);

            return Ok(())
        }

        panic!()
    }
}

impl <'a> Scope<'a> {
    pub fn resolve(&'a self, environment: Environment, name: &str) -> RResult<&'a Reference> {
        if let Some(matches) = self.references(environment).get(name) {
            return Ok(matches)
        }

        if let Some(parent) = self.parent {
            return parent.resolve(environment, name);
        }

        // take that rust, i steal ur phrasings
        Err(RuntimeError::new(format!("Cannot find '{}' in this scope", name)))
    }

    pub fn resolve_precedence_group(&self, name: &str) -> Rc<PrecedenceGroup> {
        for (group, _) in self.precedence_groups.iter() {
            if &group.name == name {
                return Rc::clone(group)
            }
        }

        panic!("Precedence group could not be resolved: {}", name)
    }
}

impl Environment {
    pub fn from_form(form: &FunctionForm) -> Environment {
        match form {
            FunctionForm::Member => Environment::Member,
            FunctionForm::Global => Environment::Global,
            FunctionForm::Constant => Environment::Global,
        }
    }
}
