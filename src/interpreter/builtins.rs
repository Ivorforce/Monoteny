use std::path::PathBuf;
use std::rc::Rc;
use crate::error::RResult;

use crate::interpreter::compiler::InlineFunction;
use crate::interpreter::opcode::{OpCode, Primitive};
use crate::interpreter::runtime::Runtime;
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
            "_write_line" => inline_fn_push(OpCode::PRINT),
            "_exit_with_error" => inline_fn_push(OpCode::PANIC),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "add" => inline_fn_push(OpCode::TRANSPILE_ADD),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.bool")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "true" => inline_fn_push_with_u8(OpCode::LOAD8, true as u8),
            "false" => inline_fn_push_with_u8(OpCode::LOAD8, false as u8),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.strings")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "add" => inline_fn_push(OpCode::ADD_STRING),
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
                compile_primitive_operation(operation, type_)
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

pub fn inline_fn_push(opcode: OpCode) -> InlineFunction {
    Rc::new(move |compiler| {{
        compiler.chunk.push(opcode);
    }})
}

pub fn inline_fn_push_with_u8(opcode: OpCode, arg: u8) -> InlineFunction {
    Rc::new(move |compiler| {{
        compiler.chunk.push_with_u8(opcode, arg);
    }})
}

pub fn compile_primitive_operation(operation: &PrimitiveOperation, type_: &primitives::Type) -> InlineFunction {
    let primitive = primitive_from_primitive(type_) as u8;

    match operation {
        PrimitiveOperation::And => inline_fn_push(OpCode::AND),
        PrimitiveOperation::Or => inline_fn_push(OpCode::OR),
        PrimitiveOperation::Not => inline_fn_push(OpCode::NOT),
        PrimitiveOperation::Negative => inline_fn_push_with_u8(OpCode::NEG, primitive),
        PrimitiveOperation::Add => inline_fn_push_with_u8(OpCode::ADD, primitive),
        PrimitiveOperation::Subtract => inline_fn_push_with_u8(OpCode::SUB, primitive),
        PrimitiveOperation::Multiply => inline_fn_push_with_u8(OpCode::MUL, primitive),
        PrimitiveOperation::Divide => inline_fn_push_with_u8(OpCode::DIV, primitive),
        PrimitiveOperation::Modulo => inline_fn_push_with_u8(OpCode::MOD, primitive),
        PrimitiveOperation::Exp => inline_fn_push_with_u8(OpCode::EXP, primitive),
        PrimitiveOperation::Log => inline_fn_push_with_u8(OpCode::LOG, primitive),
        PrimitiveOperation::EqualTo => inline_fn_push_with_u8(OpCode::EQ, primitive),
        PrimitiveOperation::NotEqualTo => inline_fn_push_with_u8(OpCode::NEQ, primitive),
        PrimitiveOperation::GreaterThan => inline_fn_push_with_u8(OpCode::GR, primitive),
        PrimitiveOperation::LesserThan => inline_fn_push_with_u8(OpCode::LE, primitive),
        PrimitiveOperation::GreaterThanOrEqual => inline_fn_push_with_u8(OpCode::GR_EQ, primitive),
        PrimitiveOperation::LesserThanOrEqual => inline_fn_push_with_u8(OpCode::LE_EQ, primitive),
        PrimitiveOperation::ParseIntString => inline_fn_push_with_u8(OpCode::PARSE, primitive),
        PrimitiveOperation::ParseRealString => inline_fn_push_with_u8(OpCode::PARSE, primitive),
        PrimitiveOperation::ToString => inline_fn_push_with_u8(OpCode::TO_STRING, primitive),
    }
}
