use std::borrow::Borrow;
use std::io::Write;
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
            write!(stream, "\n\ndef {}()", function.identifier)?;
            let return_info = function.return_type.as_ref().map(|t| self.transpile_type(&t));

            if let Some(return_info) = &return_info {
                write!(stream, " -> {}", return_info.python_type)?;
            }

            write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

            if let Some(return_info) = &return_info {
                write!(stream, "\n    Returns: {}\n", return_info.docstring_type)?;
            }

            write!(stream, "    \"\"\"\n    pass\n")?;
        }

        return Ok(())
    }

    pub fn transpile_type(&self, type_def: &Type) -> TypeInformation {
        match type_def.borrow() {
            Type::Identifier(t) => {
                match t.as_str() {
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
                        python_type: t.clone(),
                        docstring_type: t.clone()
                    }
                }
            },
            Type::NDArray(atom) => {
                match atom.as_ref() {
                    Type::Identifier(atom) => TypeInformation {
                        python_type: String::from("np.ndarray"),
                        docstring_type: String::from(format!("{}[?]", atom))
                    },
                    Type::NDArray(_) => panic!("Numpy does not support nested ndarrays.")
                }
            }
        }
    }
}
