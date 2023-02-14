use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::os::macos::raw::stat;
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, Statement};
use crate::program::global::FunctionImplementation;
use crate::program::Program;
use crate::program::primitives;


#[derive(Clone)]
pub enum Value {
    String(String),
    Primitive(primitives::Value),
}


pub fn find_main(program: &Program) -> Option<&Rc<FunctionImplementation>> {
    program.functions.iter()
        .find_or_first(|f| f.decorators.contains(&String::from("main")))
}

pub fn run_program(program: &Program, builtins: &Builtins) {
    let main_function = find_main(program).expect("No main function!");
    run_function(main_function, builtins);
}

// TODO Return Value
pub fn run_function(function: &FunctionImplementation, builtins: &Builtins) {
    let mut assignments: HashMap<Uuid, Value> = HashMap::new();

    for statement in function.statements.iter() {
        match statement.as_ref() {
            Statement::VariableAssignment(target, value) => {
                let value = evaluate(value, function, builtins, &assignments);
                assignments.insert(target.id.clone(), value.unwrap());
            }
            Statement::Expression(expression_id) => {
                evaluate(expression_id, function, builtins, &assignments);
            }
            Statement::Return(return_value) => {
                match return_value {
                    None => {
                        return;
                    }
                    Some(expression_id) => {
                        evaluate(expression_id, function, builtins, &assignments);
                        return;
                    }
                }
            }
        }
    }
}

pub fn evaluate(expression_id: &ExpressionID, function: &FunctionImplementation, builtins: &Builtins, assignments: &HashMap<Uuid, Value>) -> Option<Value> {
    let arguments = &function.expression_forest.arguments[expression_id];

    match &function.expression_forest.operations[expression_id] {
        ExpressionOperation::FunctionCall { function: fun, argument_targets, binding } => {
            if fun == &builtins.debug.print {
                let arguments_strings = arguments.iter()
                    .map(|x| evaluate(x, function, builtins, assignments).unwrap().as_string())
                    .collect_vec();

                println!("{}", arguments_strings.join(" "))
            }
            else if let Some(primitive_type) = builtins.primitives.parse_float_literal.get(fun) {
                let value = evaluate(&arguments[0], function, builtins, assignments).unwrap().as_string();
                return Some(Value::Primitive(primitive_type.parse_value(&value).unwrap()));
            }
            else if let Some(primitive_type) = builtins.primitives.parse_int_literal.get(fun) {
                let value = evaluate(&arguments[0], function, builtins, assignments).unwrap().as_string();
                return Some(Value::Primitive(primitive_type.parse_value(&value).unwrap()));
            }
            else {
                panic!()
            }
        }
        ExpressionOperation::PairwiseOperations { .. } => {
            panic!()
        }
        ExpressionOperation::VariableLookup(variable) => {
            return Some(assignments[&variable.id].clone())
        }
        ExpressionOperation::ArrayLiteral => {
            panic!()
        }
        ExpressionOperation::StringLiteral(value) => {
            return Some(Value::String(value.clone()))
        }
    }

    None
}

impl Value {
    fn as_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl Debug for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(fmt, "{}", s),
            Value::Primitive(v) => write!(fmt, "{:?}", v),
        }
    }
}
