use std::borrow::Borrow;
use std::io::Write;
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use crate::semantic_analysis::builtins::TenLangBuiltins;
use crate::semantic_analysis::computation_tree::*;

pub struct PythonTranspiler {

}

impl PythonTranspiler {
    pub fn transpile(&self, program: &Program, stream: &mut (dyn Write)) -> Result<(), std::io::Error> {
        writeln!(stream, "import numpy as np")?;

        for function in program.functions.iter() {
            write!(stream, "\n\ndef {}(", function.name)?;

            for parameter in function.parameters.iter() {
                write!(stream, "{}: ", get_external_name(&parameter))?;
                self.transpile_type(stream, &parameter.variable.type_declaration)?;
                write!(stream, ",")?;
            }

            write!(stream, ")")?;

            if let Some(return_type) = &function.return_type {
                write!(stream, " -> ", )?;
                self.transpile_type(stream, return_type)?;
            }

            write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

            if !function.parameters.is_empty() {
                write!(stream, "\n    Args:\n")?;

                for parameter in function.parameters.iter() {
                    write!(stream, "        {}: ", get_external_name(&parameter))?;
                    self.transpile_type_for_docstring(stream, &parameter.variable.type_declaration)?;
                    write!(stream, "\n")?;
                }
            }

            if let Some(return_type) = &function.return_type {
                write!(stream, "\n    Returns: ")?;
                self.transpile_type_for_docstring(stream, return_type)?;
                write!(stream, "\n")?;
            }

            write!(stream, "    \"\"\"\n")?;

            if function.statements.is_empty() {
                // No need to do conversions or anything else if we don't have a body.
                write!(stream, "    pass\n")?;
                continue
            }

            for parameter in function.parameters.iter() {
                match parameter.variable.type_declaration.borrow() {
                    Type::NDArray(atom) => {
                        if let Type::Identifier(atom) = atom.as_ref() {
                            write!(
                                stream, "    {} = np.asarray({}, dtype=",
                                parameter.variable.name,
                                get_external_name(parameter)
                            )?;
                            self.transpile_type_atom(stream, atom)?;
                            write!(stream, ")\n")?;
                        }
                        else {
                            panic!("Can't have a non-atom ndarray in numpy.")
                        }
                    },
                    _ => {
                        let external_name = get_external_name(&parameter);

                        if parameter.variable.name == external_name {
                            continue
                        }

                        writeln!(
                            stream, "    {} = {}", parameter.variable.name, external_name,
                        )?;
                    }
                }
            }

            for statement in function.statements.iter() {
                match statement.as_ref() {
                    Statement::Return(Some(expression)) => {
                        write!(stream, "    return ")?;
                        self.transpile_expression(stream, &expression, &program.builtins)?;
                    }
                    Statement::Return(None) => {
                        write!(stream, "    return")?;
                    }
                    Statement::VariableAssignment(variable, expression) => {
                        write!(stream, "    {} = ", variable.name)?;
                        self.transpile_expression(stream, expression, &program.builtins)?;
                    }
                    Statement::Expression(expression) => {
                        write!(stream, "    ")?;
                        self.transpile_expression(stream, expression, &program.builtins)?;
                    }
                }

                write!(stream, "\n")?;
            }
        }

        return Ok(())
    }

    pub fn transpile_type_atom(&self, stream: &mut (dyn Write), type_def: &String) -> Result<(), std::io::Error> {
        match type_def.as_str() {
            "Int32" => write!(stream, "np.int32")?,
            "Int64" => write!(stream, "np.int64")?,
            "Float32" => write!(stream, "np.float32")?,
            "Float64" => write!(stream, "np.float64")?,
            _ => write!(stream, "{}", type_def)?
        }

        Ok(())
    }

    pub fn transpile_type(&self, stream: &mut (dyn Write), type_def: &Type) -> Result<(), std::io::Error> {
        match type_def.borrow() {
            Type::Identifier(atom) => self.transpile_type_atom(stream, &atom)?,
            Type::NDArray(atom) => write!(stream, "np.ndarray")?,
            Type::Function(_) => todo!(),
            Type::Generic(_) => todo!(),
        }

        Ok(())
    }

    pub fn transpile_type_for_docstring(&self, stream: &mut (dyn Write), type_def: &Type) -> Result<(), std::io::Error> {
        match type_def.borrow() {
            Type::Identifier(atom) => {
                match atom.as_str() {
                    "Int32" => write!(stream, "int32")?,
                    "Int64" => write!(stream, "int64")?,
                    "Float32" => write!(stream, "float32")?,
                    "Float64" => write!(stream, "float64")?,
                    _ => write!(stream, "{}", atom)?
                }
            },
            Type::NDArray(atom) => {
                self.transpile_type_for_docstring(stream, atom)?;
                write!(stream, "[?]")?;
            },
            Type::Function(_) => todo!(),
            Type::Generic(_) => todo!(),
        }

        Ok(())
    }

    pub fn transpile_expression(&self, stream: &mut (dyn Write), expression: &Expression, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
        match &expression.operation.as_ref() {
            ExpressionOperation::Number(value ) => {
                write!(stream, "{}", value)?;
            }
            ExpressionOperation::VariableLookup(variable) => {
                write!(stream, "{}", variable.name)?;
            }
            ExpressionOperation::StaticFunctionCall { function, arguments } => {
                if self.try_transpile_binary_operator(stream, function, arguments, builtins)? {
                    // no-op
                }
                else {
                    // TODO We should make sure it calls the correct function even when shadowed.
                    write!(stream, "{}(", function.name)?;
                    for argument in arguments {
                        if let ParameterKey::String(name) = &argument.key {
                            write!(stream, "{}=", name)?;
                        }
                        self.transpile_expression(stream, &argument.value, builtins)?;
                        write!(stream, ",")?;
                    }
                    write!(stream, ")")?;
                }
            }
            ExpressionOperation::DynamicFunctionCall(_, _) => {}
            ExpressionOperation::MemberLookup(_, _) => {}
            ExpressionOperation::ArrayLiteral(_) => {}
            ExpressionOperation::StringLiteral(_) => {}
        }

        Ok(())
    }

    pub fn try_transpile_binary_operator(&self, stream: &mut (dyn Write), function: &Function, arguments: &Vec<Box<PassedArgument>>, builtins: &TenLangBuiltins) -> Result<bool, std::io::Error> {
        guard!(let [lhs, rhs] = &arguments[..] else {
            return Ok(false);
        });

        if
            function == builtins.operators.add.as_ref()
            || function == builtins.operators.subtract.as_ref()
            || function == builtins.operators.multiply.as_ref()
            || function == builtins.operators.divide.as_ref()
        {
            write!(stream, "(")?;
            self.transpile_expression(stream, &lhs.value, builtins)?;
            write!(stream, " {} ", function.name)?;
            self.transpile_expression(stream, &rhs.value, builtins)?;
            write!(stream, ")")?;
        }

        return Ok(true);
    }
}

pub fn get_external_name(parameter: &Parameter) -> String {
    match &parameter.external_key {
        // TODO This is temporary and should instead honour *, args, **, kwargs type calling
        ParameterKey::Keyless => parameter.variable.name.clone(),
        ParameterKey::String(key) => key.clone(),
    }
}
