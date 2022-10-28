use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::precedence::PrecedenceGroup;
use crate::linker::LinkError;
use crate::program::allocation::{Mutability, Reference, ReferenceType};
use crate::program::functions::{FunctionForm, FunctionOverload, FunctionPointer, HumanFunctionInterface, ParameterKey};
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement, TraitConformanceScope};
use crate::program::generics::TypeForest;
use crate::program::types::{Pattern, PatternPart, TypeProto, TypeUnit};

// Note: While a single pool cannot own overloaded variables, multiple same-level pools (-> from imports) can.
// When we have imports, this should be ignored until referenced, to avoid unnecessary import complications.
// For these cases, we could store an AMBIGUOUS value inside our pool, crashing when accessed?
type RefPool = HashMap<String, Rc<Reference>>;

#[derive(Copy, Clone, PartialEq)]
pub enum Environment {
    Global,
    Member
}

pub struct Scope<'a> {
    pub parent: Option<&'a Scope<'a>>,

    pub trait_conformance_declarations: TraitConformanceScope,

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

            trait_conformance_declarations: TraitConformanceScope::new(),
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

            trait_conformance_declarations: self.trait_conformance_declarations.clone(),
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

    pub fn overload_function(&mut self, fun: &Rc<FunctionPointer>) -> Result<(), LinkError> {
        let environment = match fun.human_interface.form {
            FunctionForm::Member => Environment::Member,
            FunctionForm::Global => Environment::Global,
            FunctionForm::Constant => Environment::Global,
        };
        let name = &fun.human_interface.name;

        let mut variables = self.references_mut(environment);

        // Remove the current FunctionOverload reference and replace with a reference containing also our new overload.
        // This may seem weird at first but it kinda makes sense - if someone queries the scope, gets a reference,
        // and then the scope is modified, the previous caller still expects their reference to not change.
        if let Some(existing) = variables.remove(name) {
            if let ReferenceType::FunctionOverload(overload) = &existing.type_ {
                let variable = Reference::make(
                    ReferenceType::FunctionOverload(overload.adding_function(fun)?)
                );

                variables.insert(fun.human_interface.name.clone(), variable);
            }
            else {
                panic!("Cannot overload with function '{}' if a variable exists in the same scope under the same name.", name);
            }
        }
        else {
            // Copy the parent's function overload into us and then add the function to the overload
            if let Some(Some(ReferenceType::FunctionOverload(overload))) = self.parent.map(|x| x.resolve(environment, name).ok().map(|x| &x.as_ref().type_)) {
                let variable = Reference::make(
                    ReferenceType::FunctionOverload(overload.adding_function(fun)?)
                );

                let mut variables = self.references_mut(environment);
                variables.insert(fun.human_interface.name.clone(), variable);
            }

            let mut variables = self.references_mut(environment);

            let variable = Reference::make(
                ReferenceType::FunctionOverload(FunctionOverload::from(fun))
            );

            variables.insert(fun.human_interface.name.clone(), variable);
        }

        Ok(())
    }

    pub fn insert_trait(&mut self, t: &Rc<Trait>) {
        let name = t.name.clone();
        self.insert_singleton(
            Environment::Global,
            Reference::make(ReferenceType::Trait(Rc::clone(t))),
            &name
        );
    }

    pub fn add_trait_conformance(&mut self, declaration: &Rc<TraitConformanceDeclaration>) {
        self.trait_conformance_declarations.add(declaration);
        for (_, pointer) in declaration.function_implementations.iter() {
            self.overload_function(pointer);
        }
        for (_, declaration) in declaration.trait_requirements_conformance.iter() {
            self.add_trait_conformance(declaration);
        }
    }

    pub fn insert_singleton(&mut self, environment: Environment, reference: Rc<Reference>, name: &String) {
        let mut references = self.references_mut(environment);

        if let Some(other) = references.insert(name.clone(), reference) {
            panic!("Multiple variables of the same name: {}", name);
        }
    }

    pub fn insert_keyword(&mut self, keyword: &String) {
        let reference = Reference::make(ReferenceType::Keyword(keyword.clone()));
        let mut references = self.references_mut(Environment::Global);

        if let Some(other) = references.insert(keyword.clone(), reference) {
            if &ReferenceType::Keyword(keyword.clone()) != &other.type_ {
                panic!("Multiple variables of the same name: {}", keyword);
            }
        }
    }

    pub fn override_variable(&mut self, environment: Environment, variable: Rc<Reference>, name: &String) {
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
    pub fn resolve(&'a self, environment: Environment, variable_name: &String) -> Result<&'a Rc<Reference>, LinkError> {
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
