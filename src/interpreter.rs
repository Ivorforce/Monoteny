use std::os::macos::raw::stat;
use std::rc::Rc;
use itertools::Itertools;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, Statement};
use crate::program::global::FunctionImplementation;
use crate::program::Program;

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
    for statement in function.statements.iter() {
        println!("{:?}", statement);

        match statement.as_ref() {
            Statement::VariableAssignment(target, value) => {
                evaluate(value, function, builtins);
                // TODO Assign
            }
            Statement::Expression(expression_id) => {
                evaluate(expression_id, function, builtins);
            }
            Statement::Return(return_value) => {
                match return_value {
                    None => {
                        return;
                    }
                    Some(expression_id) => {
                        evaluate(expression_id, function, builtins);
                        return;
                    }
                }
            }
        }
    }
}

pub fn evaluate(expression_id: &ExpressionID, function: &FunctionImplementation, builtins: &Builtins) -> Option<String> {
    let arguments = &function.expression_forest.arguments[expression_id];

    match &function.expression_forest.operations[expression_id] {
        ExpressionOperation::FunctionCall { function: fun, argument_targets, binding } => {
            if fun == &builtins.debug.print {
                let arguments_strings = arguments.iter()
                    .map(|x| evaluate(x, function, builtins).unwrap())
                    .collect_vec();

                println!("{}", arguments_strings.join(" "))
            }
            else {
                panic!()
            }
        }
        ExpressionOperation::PairwiseOperations { .. } => {
            panic!()
        }
        ExpressionOperation::VariableLookup(_) => {
            panic!()
        }
        ExpressionOperation::ArrayLiteral => {
            panic!()
        }
        ExpressionOperation::StringLiteral(value) => {
            return Some(value.clone())
        }
    }

    None
}
