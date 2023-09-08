pub mod builtins;
pub mod compiler;
pub mod run;

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use strum::IntoEnumIterator;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, Statement};
use crate::program::functions::{FunctionHead, FunctionType};
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;


pub type FunctionInterpreterImpl = Rc<dyn Fn(&mut FunctionInterpreter, ExpressionID, &RequirementsFulfillment) -> Option<Value>>;


pub struct Value {
    pub layout: Layout,
    pub data: *mut u8,
}

pub struct InterpreterGlobals {
    pub builtins: Rc<Builtins>,
    pub function_evaluators: HashMap<Uuid, FunctionInterpreterImpl>,
}

pub struct FunctionInterpreter<'a> {
    pub globals: &'a mut InterpreterGlobals,
    pub implementation: Rc<FunctionImplementation>,
    pub requirements_fulfillment: Box<RequirementsFulfillment>,

    pub assignments: HashMap<Uuid, Value>,
}

impl FunctionInterpreter<'_> {
    pub unsafe fn assign_arguments(&mut self, arguments: Vec<Value>) {
        for (arg, parameter) in zip_eq(arguments, self.implementation.parameter_variables.iter()) {
            self.assignments.insert(parameter.id.clone(), arg);
        }
    }

    pub unsafe fn run(&mut self) -> Option<Value> {
        // Avoid borrowing self.
        let self_implementation = Rc::clone(&self.implementation);
        for statement in self_implementation.statements.iter() {
            match statement.as_ref() {
                Statement::VariableAssignment(target, value) => {
                    let value = self.evaluate(*value);
                    self.assignments.insert(target.id.clone(), value.unwrap());
                }
                Statement::Expression(expression_id) => {
                    self.evaluate(*expression_id);
                }
                Statement::Return(return_value) => {
                    return match return_value {
                        None => None,
                        Some(expression_id) => self.evaluate(*expression_id),
                    }
                }
            }
        }

        return None
    }

    pub fn combine_bindings(lhs: &RequirementsFulfillment, rhs: &RequirementsFulfillment) -> Box<RequirementsFulfillment> {
        todo!()
        // Box::new(TraitResolution {
        //     requirement_bindings: lhs.requirement_bindings.iter().chain(rhs.requirement_bindings.iter())
        //         .map(|(l, r)| (Rc::clone(l), r.clone()))
        //         .collect(),
        //     function_binding: todo!(),
        // })
    }

    pub fn resolve(&self, pointer: &FunctionHead) -> Uuid {
        match &pointer.function_type {
            FunctionType::Static => pointer.function_id.clone(),
            FunctionType::Polymorphic { provided_by_assumption: requirement, abstract_function } => {
                todo!();
                // if let Some(result) = self.resolution.requirement_bindings.get(requirement).and_then(|x| x.function_binding.get(abstract_function)) {
                //     return self.resolve(&result)
                // }

                panic!("Failed to resolve abstract function: {:?}", &pointer)
            },
        }
    }

    pub unsafe fn evaluate(&mut self, expression_id: ExpressionID) -> Option<Value> {
        // TODO We should probably create an interpretation tree and an actual VM, where abstract functions are statically pre-resolved.
        //  Function instances could be assigned an int ID and thus we can call the functions directly without a UUID hash lookup. Which should be nearly as fast as a switch statement.
        //  ExpressionOperation outer switch would be replaced by having a function for every call. Literals would be stored and copied somewhere else.
        //  FunctionInterpreter instances could also be cached - no need to re-create them recursively.
        //  This would be managed by a global interpreter that is expandable dynamically. i.e. it can be re-used for interactive environments and so on.
        // Avoid borrowing self.
        let self_implementation = Rc::clone(&self.implementation);
        match &self_implementation.expression_forest.operations[&expression_id] {
            ExpressionOperation::FunctionCall(call) => {
                let function_id = self.resolve(&call.function);

                guard!(let Some(implementation) = self.globals.function_evaluators.get(&function_id) else {
                    panic!("Cannot find function ({}) with interface: {:?}", function_id, &call.function);
                });

                // Copy it to release the borrow on self.
                let implementation: FunctionInterpreterImpl = Rc::clone(&implementation);
                return implementation(self, expression_id, &call.requirements_fulfillment)
            }
            ExpressionOperation::PairwiseOperations { .. } => {
                panic!()
            }
            ExpressionOperation::VariableLookup(variable) => {
                return Some(self.assignments[&variable.id].clone())
            }
            ExpressionOperation::ArrayLiteral => {
                panic!()
            }
            ExpressionOperation::StringLiteral(value) => {
                let string_layout = Layout::new::<String>();
                let ptr = alloc(string_layout);
                *(ptr as *mut String) = value.clone();
                return Some(Value { data: ptr, layout: string_layout })
            }
        }
    }

    pub unsafe fn evaluate_arguments(&mut self, expression_id: ExpressionID) -> Vec<Value> {
        // Avoid borrowing self.
        let self_implementation = Rc::clone(&self.implementation);
        self_implementation.expression_forest.arguments[&expression_id].iter()
            .map(|x| self.evaluate(*x).unwrap())
            .collect_vec()
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data, self.layout)
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = alloc(self.layout);
            std::ptr::copy_nonoverlapping(self.data, ptr, self.layout.size());
            return Value { data: ptr, layout: self.layout }
        }
    }
}