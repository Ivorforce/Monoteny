use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::Runtime;
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


pub fn prepare(runtime: &Runtime) -> HashMap<Rc<FunctionHead>, TranspilationHint> {
    let mut transpilation_hints_by_id: HashMap<Rc<FunctionHead>, TranspilationHint> = HashMap::new();

    for ptr in runtime.source.module_by_name["math"].fn_pointers.values() {
        transpilation_hints_by_id.insert(
            Rc::clone(&ptr.target),
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

    for ptr in runtime.source.module_by_name["debug"].fn_pointers.values() {
        transpilation_hints_by_id.insert(
            Rc::clone(&ptr.target),
            TranspilationHint::CallProvided(match ptr.name.as_str() {
                "_print" => "print",
                "panic" => "exit",
                _ => continue,
            }.to_string())
        );
    }

    for ptr in runtime.source.module_by_name["strings"].fn_pointers.values() {
        transpilation_hints_by_id.insert(
            Rc::clone(&ptr.target),
            TranspilationHint::CallProvided(match ptr.name.as_str() {
                "add" => "op.add",
                _ => continue,
            }.to_string())
        );
    }

    transpilation_hints_by_id
}

pub fn try_transpile_optimization(function: &Rc<FunctionHead>, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Option<Box<Expression>> {
    if let Some(transpilation_hint) = context.fn_transpilation_hints.get(function) {
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