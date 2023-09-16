use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::Runtime;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::transpiler::python::FunctionContext;
use crate::transpiler::python::imperative::transpile_expression;
use crate::transpiler::python::ast::{Expression, Statement};

#[derive(PartialEq, Eq, Clone)]
pub enum TranspilationHint {
    Constant,
    CallProvided(String),
}


pub fn prepare<'a>(runtime: &Runtime) -> HashMap<Rc<FunctionHead>, TranspilationHint> {
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
                "_write_line" => "print",
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

pub fn optimize_implementations<'a>(transpilation_hints: &mut HashMap<Rc<FunctionHead>, TranspilationHint>, builtin_hints: &HashMap<Rc<FunctionHead>, BuiltinFunctionHint>, functions: impl Iterator<Item=&'a Box<FunctionImplementation>>) {
    'function: for function in functions {
        // TODO The !is_void avoids making 'print' a constant for now, but we should mark that function as having
        //  side effects - recursively. When we do this, we should add a way to remove all print calls deeply.
        if function.head.interface.parameters.is_empty() && !function.head.interface.return_type.unit.is_void() {
            // Could be a constant!
            for op in function.expression_forest.operations.values() {
                // TODO We should have a better way of determining whether a function is a python-builtin
                //  than 'optimizations' and 'builtin hints' - both of which might still mean anything.
                // TODO We can still make constants if they use other functions as long as we can sort them.
                match op {
                    ExpressionOperation::FunctionCall(f) => {
                        if let Some(TranspilationHint::CallProvided(_)) = transpilation_hints.get(&f.function) {
                            continue
                        }
                        else if let Some(hint) = builtin_hints.get(&f.function) {
                            match hint {
                                BuiltinFunctionHint::Constructor => continue 'function,
                                _ => {}
                            }
                        }
                        else {
                            continue 'function
                        }
                    },
                    ExpressionOperation::StringLiteral(_) => {},
                    ExpressionOperation::ArrayLiteral => {},
                    ExpressionOperation::Return => {},
                    _ => continue 'function,
                }
            }

            // Because ALL global variables are immutable, ANY other value can be a constant!
            // It just has to be copied on get - we want to do pass by value for globals anyway, TODO.
            transpilation_hints.insert(Rc::clone(&function.head), TranspilationHint::Constant);
        }
    }
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
            TranspilationHint::Constant => {
                return Some(Box::new(Expression::VariableLookup(context.names[&function.function_id].clone())))
            }
        };
    }

    None
}

pub fn try_transpile_optimized_implementation(implementation: &FunctionImplementation, context: &FunctionContext) -> Option<Box<Statement>> {
    if let Some(transpilation_hint) = context.fn_transpilation_hints.get(&implementation.head) {
        match transpilation_hint {
            TranspilationHint::Constant => {
                return Some(Box::new(Statement::VariableAssignment {
                    variable_name: context.names[&implementation.head.function_id].clone(),
                    value: transpile_expression(implementation.root_expression_id, context),
                }))
            }
            TranspilationHint::CallProvided(_) => panic!("Shouldn't try to implement builtin function!")
        }
    }

    None
}
