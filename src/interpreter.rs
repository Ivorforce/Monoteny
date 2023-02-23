pub mod builtins;
pub mod compiler;
pub mod run;

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::env::var;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use strum::IntoEnumIterator;
use crate::parser::abstract_syntax::Expression;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, Statement};
use crate::program::functions::{FunctionPointer, FunctionCallType, Function};
use crate::program::global::FunctionImplementation;
use crate::program::Program;
use crate::program::primitives;
use crate::program::traits::{TraitBinding, TraitConformanceDeclaration};


pub type FunctionInterpreterImpl = Box<dyn Fn(&mut FunctionInterpreter, &ExpressionID, &TraitBinding) -> Option<Value>>;


pub struct Value {
    pub layout: Layout,
    pub data: *mut u8,
}

pub struct FunctionInterpreter<'a> {
    pub builtins: &'a Builtins,
    pub function_evaluators: &'a HashMap<Uuid, FunctionInterpreterImpl>,

    pub implementation: &'a FunctionImplementation,
    pub binding: TraitBinding,

    pub assignments: HashMap<Uuid, Value>,
}

impl FunctionInterpreter<'_> {
    pub unsafe fn assign_arguments(&mut self, arguments: Vec<Value>) {
        // TODO Shouldn't use the human interface, but rather a set order of arguments.
        for (arg, parameter) in zip_eq(arguments, self.implementation.pointer.target.interface.parameters.iter()) {
            self.assignments.insert(parameter.target.id.clone(), arg);
        }
    }

    pub unsafe fn run(&mut self) -> Option<Value> {
        for statement in self.implementation.statements.iter() {
            match statement.as_ref() {
                Statement::VariableAssignment(target, value) => {
                    let value = self.evaluate(value);
                    self.assignments.insert(target.id.clone(), value.unwrap());
                }
                Statement::Expression(expression_id) => {
                    self.evaluate(expression_id);
                }
                Statement::Return(return_value) => {
                    return match return_value {
                        None => None,
                        Some(expression_id) => self.evaluate(expression_id),
                    }
                }
            }
        }

        return None
    }

    pub fn combine_bindings(lhs: &TraitBinding, rhs: &TraitBinding) -> TraitBinding {
        TraitBinding {
            resolution: lhs.resolution.iter().chain(rhs.resolution.iter())
                .map(|(l, r)| (Rc::clone(l), Rc::clone(r)))
                .collect()
        }
    }

    pub fn resolve(&self, pointer: &FunctionPointer) -> Uuid {
        match &pointer.call_type {
            FunctionCallType::Static => pointer.target.function_id.clone(),
            FunctionCallType::Polymorphic { requirement, abstract_function } => {
                if let Some(result) = self.binding.resolution.get(requirement).and_then(|x| x.function_binding.get(abstract_function)) {
                    return self.resolve(&result)
                }

                panic!("Failed to resolve abstract function: {:?}", &pointer)
            },
        }
    }

    pub unsafe fn evaluate(&mut self, expression_id: &ExpressionID) -> Option<Value> {
        // TODO We should probably create an interpretation tree and an actual VM, where abstract functions are statically pre-resolved.
        //  Function instances could be assigned an int ID and thus we can call the functions directly without a UUID hash lookup. Which should be nearly as fast as a switch statement.
        //  ExpressionOperation outer switch would be replaced by having a function for every call. Literals would be stored and copied somewhere else.
        //  FunctionInterpreter instances could also be cached - no need to re-create them recursively.
        //  This would be managed by a global interpreter that is expandable dynamically. i.e. it can be re-used for interactive environments and so on.

        match &self.implementation.expression_forest.operations[expression_id] {
            ExpressionOperation::FunctionCall { function: fun, binding } => {
                let function_id = self.resolve(fun);
                let implementation = &self.function_evaluators.get(&function_id);

                guard!(let Some(implementation) = implementation else {
                    panic!("Cannot find function ({}) with interface: {:?}", function_id, fun);
                });

                return implementation(self, expression_id, binding)
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

    pub unsafe fn evaluate_arguments(&mut self, expression_id: &ExpressionID) -> Vec<Value> {
        self.implementation.expression_forest.arguments[expression_id].iter()
            .map(|x| self.evaluate(x).unwrap())
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