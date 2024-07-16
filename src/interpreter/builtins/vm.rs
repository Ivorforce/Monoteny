use std::rc::Rc;
use std::path::PathBuf;
use crate::error::RResult;
use crate::interpreter::compile::function_compiler::InlineFunction;
use crate::interpreter::opcode::{OpCode, Primitive};
use crate::interpreter::runtime::Runtime;
use crate::program;
use crate::program::functions::{FunctionLogic, FunctionLogicDescriptor, PrimitiveOperation};
use crate::program::module::module_name;
use crate::program::types::TypeProto;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Monoteny files --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    runtime.repository.add("core", PathBuf::from("monoteny"));
    runtime.get_or_load_module(&module_name("core"))?;

    for function in runtime.source.module_by_name[&module_name("core.debug")].explicit_functions(&runtime.source) {
        runtime.function_inlines.insert(Rc::clone(function), match function.declared_representation.name.as_str() {
            "_write_line" => inline_fn_push(OpCode::PRINT),
            "_exit_with_error" => inline_fn_push(OpCode::PANIC),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source) {
        runtime.function_inlines.insert(Rc::clone(function), match function.declared_representation.name.as_str() {
            "add" => inline_fn_push(OpCode::TRANSPILE_ADD),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.bool")].explicit_functions(&runtime.source) {
        runtime.function_inlines.insert(Rc::clone(function), match function.declared_representation.name.as_str() {
            "true" => inline_fn_push_with_u8(OpCode::LOAD8, true as u8),
            "false" => inline_fn_push_with_u8(OpCode::LOAD8, false as u8),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.strings")].explicit_functions(&runtime.source) {
        runtime.function_inlines.insert(Rc::clone(function), match function.declared_representation.name.as_str() {
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
            FunctionLogicDescriptor::Clone(type_) => {
                if type_ == &TypeProto::unit_struct(&runtime.traits.as_ref().unwrap().String) {
                    Rc::new(move |compiler, expression| {
                        let arguments = &compiler.implementation.expression_tree.children[&expression];

                        compiler.compile_expression(&arguments[0])?;
                        todo!("String should be a type understood by us. Then we can just copy the memory region.");

                        Ok(())
                    })
                }
                else {
                    todo!()
                }
            },
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

pub fn primitive_from_primitive(primitive: &program::primitives::Type) -> Primitive {
    match primitive {
        program::primitives::Type::Bool => Primitive::BOOL,
        program::primitives::Type::Int(8) => Primitive::I8,
        program::primitives::Type::Int(16) => Primitive::I16,
        program::primitives::Type::Int(32) => Primitive::I32,
        program::primitives::Type::Int(64) => Primitive::I64,
        program::primitives::Type::UInt(8) => Primitive::U8,
        program::primitives::Type::UInt(16) => Primitive::U16,
        program::primitives::Type::UInt(32) => Primitive::U32,
        program::primitives::Type::UInt(64) => Primitive::U64,
        program::primitives::Type::Float(32) => Primitive::F32,
        program::primitives::Type::Float(64) => Primitive::F64,
        _ => todo!("Unsupported type: {:?}", primitive)
    }
}

pub fn inline_fn_push_identity() -> InlineFunction {
    Rc::new(move |compiler, expression| {{
        let arguments = &compiler.implementation.expression_tree.children[expression];
        for arg in arguments { compiler.compile_expression(arg)? }
        Ok(())
    }})
}

pub fn inline_fn_push(opcode: OpCode) -> InlineFunction {
    Rc::new(move |compiler, expression| {{
        let arguments = &compiler.implementation.expression_tree.children[expression];
        for arg in arguments { compiler.compile_expression(arg)? }

        compiler.chunk.push(opcode);
        Ok(())
    }})
}

pub fn inline_fn_push_with_u8(opcode: OpCode, arg: u8) -> InlineFunction {
    Rc::new(move |compiler, expression| {{
        let arguments = &compiler.implementation.expression_tree.children[expression];
        for arg in arguments { compiler.compile_expression(arg)? }

        compiler.chunk.push_with_u8(opcode, arg);
        Ok(())
    }})
}

pub fn compile_primitive_operation(operation: &PrimitiveOperation, type_: &program::primitives::Type) -> InlineFunction {
    let primitive = primitive_from_primitive(type_) as u8;

    match operation {
        PrimitiveOperation::And => Rc::new(move |compiler, expression| {
            let arguments = &compiler.implementation.expression_tree.children[&expression];

            // lhs
            compiler.compile_expression(&arguments[0])?;

            compiler.chunk.push(OpCode::DUP64);
            let jump_location_skip_rhs = compiler.chunk.code.len();
            compiler.chunk.push_with_u32(OpCode::JUMP_IF_FALSE, 0);

            // rhs
            compiler.compile_expression(&arguments[1])?;
            compiler.chunk.push(OpCode::AND);

            compiler.fix_jump_location_i32(jump_location_skip_rhs);

            Ok(())
        }),
        PrimitiveOperation::Or => Rc::new(move |compiler, expression| {
            let arguments = &compiler.implementation.expression_tree.children[&expression];

            // lhs
            compiler.compile_expression(&arguments[0])?;
            compiler.chunk.push(OpCode::DUP64);
            compiler.chunk.push(OpCode::NOT);

            let jump_location_skip_rhs = compiler.chunk.code.len();
            compiler.chunk.push_with_u32(OpCode::JUMP_IF_FALSE, 0);

            // rhs
            compiler.compile_expression(&arguments[1])?;
            compiler.chunk.push(OpCode::OR);

            compiler.fix_jump_location_i32(jump_location_skip_rhs);

            Ok(())
        }),
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
        PrimitiveOperation::Clone => inline_fn_push_identity(),  // Primitives are already pass-by-value.
    }
}