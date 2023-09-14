use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, Statement};
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;

pub struct ConstantFold {
    pub implementation_by_head: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,

    pub dependents: HashMap<Rc<FunctionHead>, HashSet<Rc<FunctionHead>>>,
    pub dependency_count: HashMap<Rc<FunctionHead>, i32>,

    pub forbid_inline: HashSet<Rc<FunctionHead>>,
    pub inline_hints: HashMap<Rc<FunctionHead>, InlineHint>,
}

pub enum InlineHint {
    ReplaceCall(Rc<FunctionHead>),
    YieldParameter,
    NoOp,
}

impl ConstantFold {
    pub fn new() -> ConstantFold {
        ConstantFold {
            implementation_by_head: Default::default(),
            dependents: Default::default(),
            dependency_count: Default::default(),
            forbid_inline: Default::default(),
            inline_hints: Default::default(),
        }
    }

    pub fn add(&mut self, mut implementation: Box<FunctionImplementation>, allow_inline: bool) {
        // let dependencies = HashSet::new();

        // First inline everything we can
        for i in (0 .. implementation.statements.len() - 1).rev() {
            match &implementation.statements[i].as_ref() {
                Statement::VariableAssignment(v, e) => {}
                Statement::Expression(e) => {}
                Statement::Return(Some(e)) => {}
                Statement::Return(None) => {}
            }
        }

        if allow_inline {
            if let Some(hint) = try_inline(&implementation) {
                self.inline_hints.insert(Rc::clone(&implementation.head), hint);
                self.cascade_inlines();
            }
        }
        else {
            self.forbid_inline.insert(Rc::clone(&implementation.head));
        }

        self.implementation_by_head.insert(Rc::clone(&implementation.head), implementation);
    }

    pub fn cascade_inlines(&mut self) {
        todo!();
    }

    pub fn map_call(&self, expression_id: &ExpressionID, operation: &mut ExpressionOperation) {
        match operation {
            ExpressionOperation::FunctionCall(f) => {
                if let Some(inline_hint) = self.inline_hints.get(&f.function) {
                    match inline_hint {
                        InlineHint::ReplaceCall(target_function) => {
                            *operation = ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                                function: Rc::clone(&target_function),
                                requirements_fulfillment: RequirementsFulfillment::empty(),
                            }));
                        }
                        InlineHint::YieldParameter => todo!(),
                        InlineHint::NoOp => todo!(),
                    }
                }
            }
            _ => { },
        }
    }
}

pub fn try_inline(implementation: &FunctionImplementation) -> Option<InlineHint> {
    if implementation.parameter_variables.len() != 1 || !implementation.requirements_assumption.conformance.is_empty() {
        return None;
    }

    if let [statement] = &implementation.statements[..] {
        match statement.as_ref() {
            Statement::Expression(e) => get_trivial_expression_call_target(e, implementation),
            Statement::Return(Some(e)) => get_trivial_expression_call_target(e, implementation),
            Statement::Return(None) => Some(InlineHint::NoOp),
            _ => None,
        }
    }
    else {
        None
    }
}

pub fn get_trivial_expression_call_target(expression_id: &ExpressionID, implementation: &FunctionImplementation) -> Option<InlineHint> {
    match &implementation.expression_forest.operations[expression_id] {
        ExpressionOperation::FunctionCall(f) => {
            if !f.requirements_fulfillment.is_empty() {
                return None
            }

            if let [arg] = &implementation.expression_forest.arguments[expression_id][..] {
                match &implementation.expression_forest.operations[arg] {
                    ExpressionOperation::VariableLookup(v) => {
                        if &implementation.parameter_variables[..] != [Rc::clone(v)] {
                            return None;
                        }
                    }
                    _ => return None,
                }
            }

            return Some(InlineHint::ReplaceCall(Rc::clone(&f.function)))
        },
        ExpressionOperation::VariableLookup(v) => {
            if &implementation.parameter_variables[..] == [Rc::clone(v)] {
                return Some(InlineHint::YieldParameter)
            }
        }
        _ => {},
    }

    None
}
