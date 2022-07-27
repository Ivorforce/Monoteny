use std::borrow::Borrow;
use std::io::Write;
use std::iter::zip;
use crate::abstract_syntax::Parameter;
use crate::computation_tree;
use crate::computation_tree::{Program, Type};

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

            write!(stream, "    \"\"\"\n    pass\n")?;
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
}
