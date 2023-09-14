use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use itertools::Itertools;
use crate::linker::precedence::PrecedenceGroup;
use crate::linker::LinkError;
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

    pub fn import(&mut self, module: &Module) -> Result<(), LinkError> {
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

    pub fn overload_function(&mut self, fun: &Rc<FunctionPointer>, object_ref: &Rc<ObjectReference>) -> Result<(), LinkError> {
        let name = &fun.name;
        let environment = Environment::from_form(&fun.form);

        let mut variables = self.references_mut(environment);

        // Remove the current FunctionOverload reference and replace with a reference containing also our new overload.
        // This may seem weird at first but it kinda makes sense - if someone queries the scope, gets a reference,
        // and then the scope is modified, the previous caller still expects their reference to not change.
        if let Some(existing) = variables.remove(name) {
            if let Reference::FunctionOverload(overload) = existing {
                let variable = Reference::FunctionOverload(overload.adding_function(fun, object_ref)?);

                variables.insert(fun.name.clone(), variable);
            }
            else {
                panic!("Cannot overload with function '{}' if a variable exists in the same scope under the same name.", name);
            }
        }
        else {
            // Copy the parent's function overload into us and then add the function to the overload
            if let Some(Some(Reference::FunctionOverload(overload))) = self.parent.map(|x| x.resolve(environment, name).ok()) {
                let variable = Reference::FunctionOverload(overload.adding_function(fun, object_ref)?);

                let mut variables = self.references_mut(environment);
                variables.insert(fun.name.clone(), variable);
            }

            let mut variables = self.references_mut(environment);

            let variable = Reference::FunctionOverload(FunctionOverload::from(fun, object_ref));

            variables.insert(fun.name.clone(), variable);
        }

        Ok(())
    }

    pub fn insert_singleton(&mut self, environment: Environment, reference: Reference, name: &String) {
        let mut references = self.references_mut(environment);

        if let Some(other) = references.insert(name.clone(), reference) {
            panic!("Multiple variables of the same name: {}", name);
        }
    }

    pub fn insert_keyword(&mut self, keyword: &String) {
        let reference = Reference::Keyword(keyword.clone());
        let mut references = self.references_mut(Environment::Global);

        if let Some(other) = references.insert(keyword.clone(), reference) {
            if Reference::Keyword(keyword.clone()) != other {
                panic!("Multiple variables of the same name: {}", keyword);
            }
        }
    }

    pub fn override_variable(&mut self, environment: Environment, variable: Reference, name: &String) {
        let mut variables = self.references_mut(environment);

        variables.insert(name.clone(), variable);
    }

    pub fn contains(&mut self, environment: Environment, name: &String) -> bool {
        self.references(environment).contains_key(name)
    }

    pub fn add_pattern(&mut self, pattern: Rc<Pattern>) -> Result<(), LinkError> {
        for (precedence_group, keyword_map) in self.precedence_groups.iter_mut() {
            if precedence_group != &pattern.precedence_group {
                continue;
            }

            match &pattern.parts.iter().map(|x| x.as_ref()).collect_vec()[..] {
                [_] => return Err(LinkError::LinkError { msg: format!("Pattern is too short: {}.", pattern.alias) }),
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
                _ => return Err(LinkError::LinkError { msg: String::from("This pattern form is not supported; try using unary or binary patterns.") }),
            };

            self.patterns.insert(pattern);

            return Ok(())
        }

        panic!()
    }
}

impl <'a> Scope<'a> {
    pub fn resolve(&'a self, environment: Environment, variable_name: &str) -> Result<&'a Reference, LinkError> {
        if let Some(matches) = self.references(environment).get(variable_name) {
            return Ok(matches)
        }

        if let Some(parent) = self.parent {
            return parent.resolve(environment, variable_name);
        }

        Err(LinkError::LinkError { msg: format!("Variable '{}' could not be resolved", variable_name) })
    }

    pub fn resolve_precedence_group(&self, name: &String) -> Rc<PrecedenceGroup> {
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
