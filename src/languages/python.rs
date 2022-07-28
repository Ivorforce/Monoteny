use std::borrow::Borrow;
use std::io::Write;
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use crate::abstract_syntax::Parameter;
use crate::semantic_analysis::builtins::TenLangBuiltins;
use crate::semantic_analysis::computation_tree::*;

pub struct PythonTranspiler {

}

struct TypeInformation {
    python_type: String,
    docstring_type: String
}

impl PythonTranspiler {
    pub fn transpile(&self, program: &Program, stream: &mut (dyn Write)) -> Result<(), std::io::Error> {
        writeln!(stream, "import numpy as np")?;

        for function in program.functions.iter() {
            let return_info = function.return_type.as_ref()
                .map(|t| self.transpile_type(&t));

            let parameters_type_info: Vec<TypeInformation> = function.parameters.iter()
                .map(|x| self.transpile_type(&x.variable.type_declaration))
                .collect();

            write!(stream, "\n\ndef {}(", function.identifier)?;

            for (parameter, type_info) in zip(function.parameters.iter(), parameters_type_info.iter()) {
                write!(stream, "{}: {},", parameter.external_name, type_info.python_type)?;
            }

            write!(stream, ")")?;

            if let Some(return_info) = &return_info {
                write!(stream, " -> {}", return_info.python_type)?;
            }

            write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

            if !function.parameters.is_empty() {
                write!(stream, "\n    Args:\n")?;

                for (parameter, type_info) in zip(function.parameters.iter(), parameters_type_info.iter()) {
                    write!(stream, "        {}: {}", parameter.external_name, type_info.docstring_type)?;
                }

                write!(stream, "\n")?;
            }

            if let Some(return_info) = &return_info {
                write!(stream, "\n    Returns: {}\n", return_info.docstring_type)?;
            }

            write!(stream, "    \"\"\"\n")?;

            if function.statements.is_empty() {
                // No need to do conversions or anything else if we don't have a body.
                write!(stream, "    pass\n")?;
                continue
            }

            for (parameter, type_info) in zip(function.parameters.iter(), parameters_type_info.iter()) {
                match parameter.variable.type_declaration.borrow() {
                    Type::NDArray(atom) => {
                        // Can't be ndarray at this point
                        if let Type::Identifier(atom) = atom.as_ref() {
                            writeln!(
                                stream, "    {} = np.asarray({}, dtype={}),",
                                parameter.variable.name, parameter.external_name, self.transpile_atom_type(atom.as_str()).python_type
                            )?;
                        }
                    },
                    _ => {
                        if parameter.variable.name == parameter.external_name {
                            continue
                        }

                        writeln!(
                            stream, "    {} = {}", parameter.variable.name, parameter.external_name,
                        )?;
                    }
                }
            }

            for statement in function.statements.iter() {
                match statement.as_ref() {
                    Statement::Return(Some(expression)) => {
                        write!(stream, "    return ")?;
                        self.transpile_expression(stream, &expression, &program.builtins)?;
                        write!(stream, "\n")?;
                    }
                    Statement::Return(None) => {
                        write!(stream, "    return\n")?;
                    }
                    _ => todo!()
                }
            }
        }

        return Ok(())
    }

    pub fn transpile_atom_type(&self, type_def: &str) -> TypeInformation {
        match type_def {
            "Int32" => TypeInformation {
                python_type: String::from("np.int32"),
                docstring_type: String::from("int32")
            },
            "Int64" => TypeInformation {
                python_type: String::from("np.int64"),
                docstring_type: String::from("int64")
            },
            "Float32" => TypeInformation {
                python_type: String::from("np.float32"),
                docstring_type: String::from("float32")
            },
            "Float64" => TypeInformation {
                python_type: String::from("np.float64"),
                docstring_type: String::from("float64")
            },
            _ => TypeInformation {
                python_type: String::from(type_def),
                docstring_type: String::from(type_def)
            }
        }
    }

    pub fn transpile_type(&self, type_def: &Type) -> TypeInformation {
        match type_def.borrow() {
            Type::Identifier(atom) => self.transpile_atom_type(atom.as_str()),
            Type::NDArray(atom) => {
                match atom.as_ref() {
                    Type::Identifier(atom) => TypeInformation {
                        python_type: String::from("np.ndarray"),
                        docstring_type: String::from(format!("{}[?]", self.transpile_atom_type(atom.as_str()).docstring_type))
                    },
                    Type::NDArray(_) => panic!("Numpy does not support nested ndarrays.")
                }
            }
        }
    }

    pub fn transpile_expression(&self, stream: &mut (dyn Write), expression: &Expression, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
        Ok(match &expression.operation.as_ref() {
            ExpressionOperation::VariableLookup(variable) => {
                write!(stream, "{}", variable.name)?;
            }
            ExpressionOperation::FunctionCall(function, arguments) => {
                if self.try_transpile_binary_operator(stream, function, arguments, builtins)? {
                    // no-op
                }
                else {
                    self.transpile_expression(stream, function, builtins)?;
                    write!(stream, "(")?;
                    for argument in arguments {
                        if let Some(name) = &argument.name {
                            write!(stream, "{}=", name)?;
                        }
                        self.transpile_expression(stream, &argument.value, builtins)?;
                        write!(stream, ",")?;
                    }
                    write!(stream, ")")?;
                }
            }
            _ => todo!()
        })
    }

    pub fn try_transpile_binary_operator(&self, stream: &mut (dyn Write), function: &Box<Expression>, arguments: &Vec<Box<PassedArgument>>, builtins: &TenLangBuiltins) -> Result<bool, std::io::Error> {
        guard!(let ExpressionOperation::VariableLookup(variable) = function.operation.as_ref() else {
            return Ok(false);
        });
        guard!(let [lhs, rhs] = &arguments[..] else {
            return Ok(false);
        });

        if variable == &builtins.operators.add || variable == &builtins.operators.subtract || variable == &builtins.operators.multiply || variable == &builtins.operators.divide {
            write!(stream, "(")?;
            self.transpile_expression(stream, &lhs.value, builtins)?;
            write!(stream, " {} ", variable.name)?;
            self.transpile_expression(stream, &rhs.value, builtins)?;
            write!(stream, ")")?;
        }

        return Ok(true);
    }
}
