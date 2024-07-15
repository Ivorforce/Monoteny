#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FunctionCallExplicity {
    Explicit,
    Implicit,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FunctionTargetType {
    Global,
    Member
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FunctionRepresentation {
    /// Name of the function.
    pub name: String,
    pub target_type: FunctionTargetType,
    pub call_explicity: FunctionCallExplicity,
}


impl FunctionRepresentation {
    pub fn new(name: &str, target_type: FunctionTargetType, explicity: FunctionCallExplicity) -> FunctionRepresentation {
        FunctionRepresentation {
            name: name.into(),
            target_type,
            call_explicity: explicity,
        }
    }

    pub fn new_global_function(name: &str) -> FunctionRepresentation {
        FunctionRepresentation {
            name: name.into(),
            target_type: FunctionTargetType::Global,
            call_explicity: FunctionCallExplicity::Explicit,
        }
    }

    pub fn new_global_implicit(name: &str) -> FunctionRepresentation {
        FunctionRepresentation {
            name: name.into(),
            target_type: FunctionTargetType::Global,
            call_explicity: FunctionCallExplicity::Implicit,
        }
    }
    pub fn new_member_function(name: &str) -> FunctionRepresentation {
        FunctionRepresentation {
            name: name.into(),
            target_type: FunctionTargetType::Member,
            call_explicity: FunctionCallExplicity::Explicit,
        }
    }

    pub fn new_member_implicit(name: &str) -> FunctionRepresentation {
        FunctionRepresentation {
            name: name.into(),
            target_type: FunctionTargetType::Member,
            call_explicity: FunctionCallExplicity::Implicit,
        }
    }
}
