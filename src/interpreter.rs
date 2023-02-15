mod builtins;

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::os::macos::raw::stat;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use strum::IntoEnumIterator;
use crate::parser::abstract_syntax::Expression;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, Statement};
use crate::program::functions::FunctionPointer;
use crate::program::global::FunctionImplementation;
use crate::program::Program;
use crate::program::primitives;


pub type FunctionInterpreterImpl = Box<dyn Fn(&mut FunctionInterpreter, &ExpressionID) -> Option<Value>>;


#[derive(Clone)]
pub struct Value {
    pub layout: Layout,
    pub data: *mut u8,
}

pub struct FunctionInterpreter<'a> {
    pub builtins: &'a Builtins,
    pub function_evaluators: &'a HashMap<Rc<FunctionPointer>, FunctionInterpreterImpl>,

    pub function: &'a FunctionImplementation,

    pub assignments: HashMap<Uuid, Value>,
}

pub fn find_main(program: &Program) -> Option<&Rc<FunctionImplementation>> {
    program.functions.iter()
        .find_or_first(|f| f.decorators.contains(&String::from("main")))
}

pub fn run_program(program: &Program, builtins: &Builtins) {
    let main_function = find_main(program).expect("No main function!");
    let mut interpreter = FunctionInterpreter {
        builtins,
        function_evaluators: &builtins::make_evaluators(builtins),
        function: main_function,
        assignments: HashMap::new(),
    };
    unsafe {
        interpreter.run();
    }
}

impl FunctionInterpreter<'_> {
    pub unsafe fn run(&mut self) -> Option<Value> {
        for statement in self.function.statements.iter() {
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

    pub unsafe fn evaluate(&mut self, expression_id: &ExpressionID) -> Option<Value> {
        match &self.function.expression_forest.operations[expression_id] {
            ExpressionOperation::FunctionCall { function: fun, argument_targets, binding } => {
                // TODO Resolve actual function via our binding (not the one FOR the function)
                let implementation = &self.function_evaluators.get(fun);

                guard!(let Some(implementation) = implementation else {
                    panic!("Cannot find implementation for function: {:?}", &fun.human_interface);
                });

                return implementation(self, expression_id)
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

    pub unsafe fn evaluate_vec(&mut self, expression_id: &ExpressionID) -> Vec<Value> {
        self.function.expression_forest.arguments[expression_id].iter()
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
