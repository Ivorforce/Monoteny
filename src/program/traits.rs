use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use itertools::Itertools;

pub use binding::TraitBinding;
pub use conformance::{RequirementsAssumption, RequirementsFulfillment, TraitConformance, TraitConformanceWithTail};
pub use graph::{TraitConformanceRule, TraitGraph};
pub use structs::StructInfo;
pub use trait_::{FieldHint, Trait};

mod conformance;
mod binding;
mod graph;
mod trait_;
mod structs;
