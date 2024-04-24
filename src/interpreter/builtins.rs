use std::path::PathBuf;
use std::rc::Rc;

use crate::error::RResult;
use crate::interpreter::chunks::{OpCode, Primitive};
use crate::interpreter::compiler::FunctionCompiler;
use crate::interpreter::Runtime;
use crate::program::global::{FunctionLogic, FunctionLogicDescriptor, PrimitiveOperation};
use crate::program::module::module_name;
use crate::program::primitives;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Monoteny files --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    runtime.repository.add("core", PathBuf::from("monoteny"));
    runtime.get_or_load_module(&module_name("core"))?;

    for function in runtime.source.module_by_name[&module_name("core.debug")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "_write_line" => Rc::new(move |compiler| {{
                compiler.chunk.push(OpCode::PRINT);
            }}),
            "_exit_with_error" => Rc::new(move |compiler| {{
                panic!();
            }}),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "add" => Rc::new(move |compiler| {{
                compiler.chunk.push(OpCode::TRANSPILE_ADD);
            }}),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.bool")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "true" => Rc::new(move |compiler| {{
                compiler.chunk.push_with_u8(OpCode::TRANSPILE_ADD, true as u8);
            }}),
            "false" => Rc::new(move |compiler| {{
                compiler.chunk.push_with_u8(OpCode::TRANSPILE_ADD, false as u8);
            }}),
            _ => continue,
        });
    }

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Math --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    for function in runtime.source.module_by_name[&module_name("builtins")].explicit_functions(&runtime.source) {
        let Some(FunctionLogic::Descriptor(descriptor)) = runtime.source.fn_logic.get(function) else {
            continue;
        };

        runtime.function_inlines.insert(Rc::clone(function), match descriptor {
            FunctionLogicDescriptor::Stub => todo!(),
            FunctionLogicDescriptor::TraitProvider(_) => continue,
            FunctionLogicDescriptor::FunctionProvider(_) => continue,
            FunctionLogicDescriptor::PrimitiveOperation { type_, operation } => {
                let Some(opcode) = opcode_from_primitive_operation(operation) else {
                    continue
                };

                let primitive = primitive_from_primitive(type_);
                Rc::new(move |compiler| {{
                    compiler.chunk.push_with_u8(opcode, primitive as u8);
                }})
            }
            FunctionLogicDescriptor::Constructor(_) => todo!(),
            FunctionLogicDescriptor::GetMemberField(_, _) => todo!(),
            FunctionLogicDescriptor::SetMemberField(_, _) => todo!(),
        });
    }

    Ok(())
}

pub fn primitive_from_primitive(primitive: &primitives::Type) -> Primitive {
    match primitive {
        primitives::Type::Bool => Primitive::BOOL,
        primitives::Type::Int(8) => Primitive::I8,
        primitives::Type::Int(16) => Primitive::I16,
        primitives::Type::Int(32) => Primitive::I32,
        primitives::Type::Int(64) => Primitive::I64,
        primitives::Type::UInt(8) => Primitive::U8,
        primitives::Type::UInt(16) => Primitive::U16,
        primitives::Type::UInt(32) => Primitive::U32,
        primitives::Type::UInt(64) => Primitive::U64,
        primitives::Type::Float(32) => Primitive::F32,
        primitives::Type::Float(64) => Primitive::F64,
        _ => todo!("Unsupported type: {:?}", primitive)
    }
}

pub fn opcode_from_primitive_operation(operation: &PrimitiveOperation) -> Option<OpCode> {
    // TODO The empty ones should be filled in
    Some(match operation {
        PrimitiveOperation::And => OpCode::AND,
        PrimitiveOperation::Or => OpCode::OR,
        PrimitiveOperation::Not => return None,
        PrimitiveOperation::Negative => return None,
        PrimitiveOperation::Add => OpCode::ADD,
        PrimitiveOperation::Subtract => OpCode::SUB,
        PrimitiveOperation::Multiply => OpCode::MUL,
        PrimitiveOperation::Divide => OpCode::DIV,
        PrimitiveOperation::Modulo => return None,
        PrimitiveOperation::Exp => return None,
        PrimitiveOperation::Log => return None,
        PrimitiveOperation::EqualTo => OpCode::EQ,
        PrimitiveOperation::NotEqualTo => OpCode::NEQ,
        PrimitiveOperation::GreaterThan => OpCode::GR,
        PrimitiveOperation::LesserThan => OpCode::LE,
        PrimitiveOperation::GreaterThanOrEqual => OpCode::GR_EQ,
        PrimitiveOperation::LesserThanOrEqual => OpCode::LE_EQ,
        PrimitiveOperation::ParseIntString => return None,
        PrimitiveOperation::ParseRealString => return None,
        PrimitiveOperation::ToString => return None,
    })
}
