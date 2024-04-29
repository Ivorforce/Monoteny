use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use itertools::Itertools;

pub use crate::program::traits::binding::TraitBinding;
pub use crate::program::traits::conformance::{TraitConformance, TraitConformanceWithTail, RequirementsAssumption, RequirementsFulfillment};
pub use crate::program::traits::graph::{TraitConformanceRule, TraitGraph};
pub use crate::program::traits::trait_::{FieldHint, Trait};

mod conformance;
mod binding;
mod graph;
mod trait_;
