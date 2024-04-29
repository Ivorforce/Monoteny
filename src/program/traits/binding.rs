use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use itertools::Itertools;

use crate::program::generics::GenericAlias;
use crate::program::traits::Trait;
use crate::program::types::TypeProto;
use crate::util::fmt::write_keyval;

/// Some application of a trait with specific types.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitBinding {
    /// The trait that is bound.
    pub trait_: Rc<Trait>,

    /// A mapping from each of the trait's generics to some type.
    pub generic_to_type: HashMap<Rc<Trait>, Rc<TypeProto>>,
}

impl TraitBinding {
    pub fn mapping_types(&self, map: &dyn Fn(&Rc<TypeProto>) -> Rc<TypeProto>) -> Rc<TraitBinding> {
        Rc::new(TraitBinding {
            trait_: Rc::clone(&self.trait_),
            generic_to_type: self.generic_to_type.iter().map(|(generic, type_) | (Rc::clone(generic), map(type_))).collect()
        })
    }

    pub fn try_mapping_types<B>(&self, map: &dyn Fn(&Rc<TypeProto>) -> Result<Rc<TypeProto>, B>) -> Result<Rc<TraitBinding>, B> {
        Ok(Rc::new(TraitBinding {
            trait_: Rc::clone(&self.trait_),
            generic_to_type: self.generic_to_type.iter().map(|(generic, type_) | Ok((Rc::clone(generic), map(type_)?))).try_collect()?
        }))
    }

    pub fn collect_generics(&self) -> HashSet<GenericAlias> {
        TypeProto::collect_generics(self.generic_to_type.values())
    }
}

impl Hash for TraitBinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.trait_.hash(state);

        for keyval in self.generic_to_type.iter().sorted_by_key(|(trait_, type_)| trait_.id) {
            keyval.hash(state);
        }
    }
}

impl Debug for TraitBinding {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.trait_.name)?;
        if !self.generic_to_type.is_empty() {
            write!(fmt, "<")?;
            write_keyval(fmt, &self.generic_to_type)?;
            write!(fmt, ">")?;
        }
        Ok(())
    }
}
