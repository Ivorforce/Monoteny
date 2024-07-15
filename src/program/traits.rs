use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use itertools::Itertools;

pub use binding::TraitBinding;
pub use conformance::{RequirementsAssumption, RequirementsFulfillment, TraitConformance, TraitConformanceWithTail};
pub use graph::{TraitConformanceRule, TraitGraph};
pub use trait_::{FieldHint, Trait};
pub use structs::StructInfo;

mod conformance;
mod binding;
mod graph;
mod trait_;
mod structs;
