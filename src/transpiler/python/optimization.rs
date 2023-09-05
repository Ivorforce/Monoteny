use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::ExpressionID;
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::transpiler::python::FunctionContext;
use crate::transpiler::python::imperative::transpile_expression;
use crate::transpiler::python::tree::Expression;

#[derive(PartialEq, Eq, Clone)]
pub enum TranspilationHint {
    CallProvided(String)
}

const MATH_REPLACEMENTS: [(&str, &str); 17] = [
    ("factorial", "math.factorial"),

    ("sin", "math.sin"),
    ("cos", "math.cos"),
    ("tan", "math.tan"),
    ("sinh", "math.sinh"),
    ("cosh", "math.cosh"),
    ("tanh", "math.tanh"),
    ("arcsin", "math.asin"),
    ("arccos", "math.acos"),
    ("arctan", "math.atan"),
    ("arcsinh", "math.asinh"),
    ("arccosh", "math.acosh"),
    ("arctanh", "math.atanh"),

    ("ceil", "math.ceil"),
    ("floor", "math.floor"),
    ("round", "round"),

    ("abs", "abs"),
];

pub fn prepare(builtins: &Builtins) -> HashMap<Uuid, TranspilationHint> {
    let mut transpilation_hints_by_id: HashMap<Uuid, TranspilationHint> = HashMap::new();

    let math_replacements: HashMap<_, _> = MATH_REPLACEMENTS.iter()
        .map(|(src, dst)| (src.to_string(), dst.to_string()))
        .collect();
    for ptr in builtins.module_by_name["math".into()].function_pointers.values() {
        if let Some(fn_name) = math_replacements.get(&ptr.name) {
            let id: Uuid = ptr.target.function_id;
            let hint: TranspilationHint = TranspilationHint::CallProvided(fn_name.clone());
            transpilation_hints_by_id.insert(id, hint);
        }
    }

    transpilation_hints_by_id
}

pub fn try_transpile_optimization(function: &Rc<FunctionHead>, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Option<Box<Expression>> {
    if !context.builtins.module_by_name["math".into()].functions_references.contains_key(function) {
        return None
    }

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