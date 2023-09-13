use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::ExpressionID;
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::transpiler::python::FunctionContext;
use crate::transpiler::python::imperative::transpile_expression;
use crate::transpiler::python::ast::Expression;

#[derive(PartialEq, Eq, Clone)]
pub enum TranspilationHint {
    CallProvided(String)
}


pub fn prepare(builtins: &Builtins) -> HashMap<Uuid, TranspilationHint> {
    let mut transpilation_hints_by_id: HashMap<Uuid, TranspilationHint> = HashMap::new();

    for ptr in builtins.module_by_name["math".into()].function_pointers.values() {
        transpilation_hints_by_id.insert(
            ptr.target.function_id,
            TranspilationHint::CallProvided(match ptr.name.as_str() {
                "factorial" => "math.factorial",

                "sin" => "math.sin",
                "cos" => "math.cos",
                "tan" => "math.tan",
                "sinh" => "math.sinh",
                "cosh" => "math.cosh",
                "tanh" => "math.tanh",
                "arcsin" => "math.asin",
                "arccos" => "math.acos",
                "arctan" => "math.atan",
                "arcsinh" => "math.asinh",
                "arccosh" => "math.acosh",
                "arctanh" => "math.atanh",

                "ceil" => "math.ceil",
                "floor" => "math.floor",
                "round" => "round",

                "abs" => "abs",
                _ => continue,
            }.to_string())
        );
    }

    for ptr in builtins.module_by_name["debug".into()].function_pointers.values() {
        transpilation_hints_by_id.insert(
            ptr.target.function_id,
            TranspilationHint::CallProvided(match ptr.name.as_str() {
                "print" => "print",
                "panic" => "exit",
                _ => continue,
            }.to_string())
        );
    }

    for ptr in builtins.module_by_name["strings".into()].function_pointers.values() {
        transpilation_hints_by_id.insert(
            ptr.target.function_id,
            TranspilationHint::CallProvided(match ptr.name.as_str() {
                "add" => "op.add",
                _ => continue,
            }.to_string())
        );
    }

    transpilation_hints_by_id
}

pub fn try_transpile_optimization(function: &Rc<FunctionHead>, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Option<Box<Expression>> {
    if let Some(transpilation_hint) = context.transpilation_hints.get(&function.function_id) {
        match transpilation_hint {
            TranspilationHint::CallProvided(python_name) => {
                return Some(
                    Box::new(Expression::FunctionCall(
                        python_name.clone(),
                        arguments.iter()
                            .map(|expression| (ParameterKey::Positional, transpile_expression(*expression, context)))
                            .collect())
                    )
                )
            }
        };
    }

    None
}