use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use uuid::Uuid;

use crate::program::functions::FunctionHead;
use crate::program::traits::TraitBinding;
use crate::program::types::TypeProto;
use crate::util::fmt::write_separated_display;

/// Which role a `Nominal` plays. All three share the same machinery (conformance,
/// generics, requirements); the kind decides how a bare use of the name resolves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NominalKind {
    /// An interface (the `trait` keyword). A bare use in a signature invents a generic
    /// conforming to it.
    Trait,
    /// A concrete type (the `struct` keyword, builtin primitives, String). A bare use is
    /// the type itself, never a generic.
    Struct,
    /// A generic placeholder: `Self`, invented generics, and anonymous `#`/`#A`. Resolves
    /// to itself.
    Generic,
}

/// The definition of a named type — a trait, a struct, or a generic placeholder (see `kind`).
#[derive(Clone)]
pub struct Nominal {
    pub id: Uuid,
    pub name: String,
    pub kind: NominalKind,

    // Generics declared for this type, by name (via its declaration).
    // Used in abstract functions and requirements (collect_generics on those would yield the same GenericAliases).
    pub generics: HashMap<String, Rc<Nominal>>,

    // To conform to this type, these other conformances are required.
    pub requirements: HashSet<Rc<TraitBinding>>,

    // Functions required by this type specifically (not its requirements).
    pub abstract_functions: HashSet<Rc<FunctionHead>>,
    pub field_hints: Vec<FieldHint>,
}


/// Information about a field declared on a named type. For a `struct` these become the
///  constructor parameters and accessors; for a `trait` they are requirements on conformers.
#[derive(Clone)]
pub struct FieldHint {
    pub name: String,
    pub type_: Rc<TypeProto>,
    pub setter: Option<Rc<FunctionHead>>,
    pub getter: Option<Rc<FunctionHead>>,
}

impl Nominal {
    /// A generic placeholder with no `Self` of its own (`Self`, invented generics, `#A`).
    pub fn new_flat(name: &str) -> Nominal {
        Nominal {
            id: Uuid::new_v4(),
            name: name.to_string(),
            kind: NominalKind::Generic,
            generics: Default::default(),
            requirements: Default::default(),
            abstract_functions: Default::default(),
            field_hints: Default::default(),
        }
    }

    /// An interface (`trait`). Its requirements are delivered by conforming types.
    pub fn new_trait(name: &str) -> Nominal {
        Nominal::new_self_kind(name, NominalKind::Trait)
    }

    /// A concrete type (`struct`, builtin primitives, String). Instantiable, never abstract.
    pub fn new_struct(name: &str) -> Nominal {
        Nominal::new_self_kind(name, NominalKind::Struct)
    }

    fn new_self_kind(name: &str, kind: NominalKind) -> Nominal {
        Nominal {
            id: Uuid::new_v4(),
            name: name.to_string(),
            kind,
            generics: HashMap::from([("Self".to_string(), Rc::new(Nominal::new_flat("Self")))]),
            requirements: Default::default(),
            abstract_functions: Default::default(),
            field_hints: Default::default(),
        }
    }

    pub fn create_generic_type(self: &Nominal, generic_name: &str) -> Rc<TypeProto> {
        TypeProto::unit_struct(&self.generics[generic_name])
    }

    pub fn create_generic_binding(self: &Rc<Nominal>, generic_to_type: Vec<(&str, Rc<TypeProto>)>) -> Rc<TraitBinding> {
        Rc::new(TraitBinding {
            trait_: Rc::clone(self),
            generic_to_type: HashMap::from_iter(
                generic_to_type.into_iter()
                    .map(|(generic_name, type_)| (Rc::clone(&self.generics[generic_name]), type_))
            ),
        })
    }

    pub fn add_simple_parent_requirement(&mut self, parent_trait: &Rc<Nominal>) {
        self.requirements.insert(
            parent_trait.create_generic_binding(vec![("Self", self.create_generic_type("Self"))])
        );
    }
}

impl PartialEq for Nominal {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Nominal {}

impl Hash for Nominal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for Nominal {
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
