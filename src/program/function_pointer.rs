use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use display_with_options::with_options;

use crate::program::function_object::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
use crate::program::functions::{FunctionHead, FunctionInterface};

pub struct FunctionPointer {
    pub target: Rc<FunctionHead>,
    pub representation: FunctionRepresentation,
}

impl FunctionPointer {
    pub fn new_global_function(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new_static(interface),
            representation: FunctionRepresentation {
                name: name.to_string(),
                target_type: FunctionTargetType::Global,
                call_explicity: FunctionCallExplicity::Explicit,
            },
        })
    }

    pub fn new_member_function(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new_static(interface),
            representation: FunctionRepresentation {
                name: name.to_string(),
                target_type: FunctionTargetType::Member,
                call_explicity: FunctionCallExplicity::Explicit,
            },
        })
    }

    pub fn new_member_implicit(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new_static(interface),
            representation: FunctionRepresentation {
                name: name.to_string(),
                target_type: FunctionTargetType::Member,
                call_explicity: FunctionCallExplicity::Implicit,
            },
        })
    }

    pub fn new_global_implicit(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new_static(interface),
            representation: FunctionRepresentation {
                name: name.to_string(),
                target_type: FunctionTargetType::Global,
                call_explicity: FunctionCallExplicity::Implicit,
            },
        })
    }
}

impl Debug for FunctionPointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", with_options(self.target.as_ref(), &self.representation))
    }
}
