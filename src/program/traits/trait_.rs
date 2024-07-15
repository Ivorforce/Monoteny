use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use uuid::Uuid;

use crate::program::functions::FunctionHead;
use crate::program::traits::TraitBinding;
use crate::program::types::TypeProto;
use crate::util::fmt::write_separated_display;

/// The definition of some trait.
#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    // Generics declared for this trait, by name (via its declaration).
    // Used in abstract functions and requirements (collect_generics on those would yield the same GenericAliases).
    pub generics: HashMap<String, Rc<Trait>>,

    // To conform to this trait, these other conformances are required.
    pub requirements: HashSet<Rc<TraitBinding>>,

    // Functions required by this trait specifically (not its requirements).
    pub abstract_functions: HashSet<Rc<FunctionHead>>,
    pub field_hints: Vec<FieldHint>,
}


/// For traits, information about certain fields that have been declared.
///  This is useful mostly if somebody wants to instantiate the trait without sub-traiting it.
#[derive(Clone)]
pub struct FieldHint {
    pub name: String,
    pub type_: Rc<TypeProto>,
    pub setter: Option<Rc<FunctionHead>>,
    pub getter: Option<Rc<FunctionHead>>,
}

impl Trait {
    pub fn new_flat(name: &str) -> Trait {
        Trait {
            id: Uuid::new_v4(),
            name: name.to_string(),
            generics: Default::default(),
            requirements: Default::default(),
            abstract_functions: Default::default(),
            field_hints: Default::default(),
        }
    }

    pub fn new_with_self(name: &str) -> Trait {
        Trait {
            id: Uuid::new_v4(),
            name: name.to_string(),
            generics: HashMap::from([("Self".to_string(), Rc::new(Trait::new_flat("Self")))]),
            requirements: Default::default(),
            abstract_functions: Default::default(),
            field_hints: Default::default(),
        }
    }

    pub fn create_generic_type(self: &Trait, generic_name: &str) -> Rc<TypeProto> {
        TypeProto::unit_struct(&self.generics[generic_name])
    }

    pub fn create_generic_binding(self: &Rc<Trait>, generic_to_type: Vec<(&str, Rc<TypeProto>)>) -> Rc<TraitBinding> {
        Rc::new(TraitBinding {
            trait_: Rc::clone(self),
            generic_to_type: HashMap::from_iter(
                generic_to_type.into_iter()
                    .map(|(generic_name, type_)| (Rc::clone(&self.generics[generic_name]), type_))
            ),
        })
    }

    pub fn add_simple_parent_requirement(&mut self, parent_trait: &Rc<Trait>) {
        self.requirements.insert(
            parent_trait.create_generic_binding(vec![("Self", self.create_generic_type("Self"))])
        );
    }
}

impl PartialEq for Trait {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Trait {}

impl Hash for Trait {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for Trait {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}<{}>", self.name, self.id)?;
        if !self.generics.is_empty() {
            write!(fmt, "<")?;
            write_separated_display(fmt, ", ", self.generics.keys())?;
            write!(fmt, ">")?;
        }
        Ok(())
    }
}
