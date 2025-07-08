use std::mem::transmute;
use std::rc::Rc;
use monoteny_macro::un_expr;
use crate::error::RResult;
use crate::interpreter::compile::function_compiler::InlineFunction;
use crate::interpreter::data::string_to_ptr;
use crate::interpreter::opcode::{OpCode, Primitive};
use crate::interpreter::opcode::OpCode::CALL_INTRINSIC;
use crate::interpreter::runtime::Runtime;
use crate::interpreter::vm::IntrinsicFunction;
use crate::program;
use crate::program::functions::{FunctionLogic, FunctionLogicDescriptor, PrimitiveOperation};
use crate::program::module::module_name;
use crate::program::types::TypeProto;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Monoteny files --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    for function in runtime.source.module_by_name[&module_name("core.debug")].explicit_functions(&runtime.source) {
        runtime.compile_server.function_inlines.insert(function.function_id, match function.declared_representation.name.as_str() {
            "_write_line" => inline_fn_push(OpCode::PRINT),
            "_exit_with_error" => inline_fn_push(OpCode::PANIC),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source) {
        runtime.compile_server.function_inlines.insert(function.function_id, match function.declared_representation.name.as_str() {
            "add" => inline_fn_push(OpCode::TRANSPILE_ADD),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.bool")].explicit_functions(&runtime.source) {
        runtime.compile_server.function_inlines.insert(function.function_id, match function.declared_representation.name.as_str() {
            "true" => inline_fn_push_with_u8(OpCode::LOAD8, true as u8),
            "false" => inline_fn_push_with_u8(OpCode::LOAD8, false as u8),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.strings")].explicit_functions(&runtime.source) {
        runtime.compile_server.function_inlines.insert(function.function_id, match function.declared_representation.name.as_str() {
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

        runtime.compile_server.function_inlines.insert(function.function_id, match descriptor {
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

pub fn inline_fn_push_with_u64(opcode: OpCode, arg: u64) -> InlineFunction {
    Rc::new(move |compiler, expression| {{
        let arguments = &compiler.implementation.expression_tree.children[expression];
        for arg in arguments { compiler.compile_expression(arg)? }

        compiler.chunk.push_with_u64(opcode, arg);
        Ok(())
    }})
}

pub unsafe fn to_str_ptr<A: ToString>(a: A) -> *mut () {
    let string = a.to_string();
    string_to_ptr(&string)
}

pub fn compile_primitive_operation(operation: &PrimitiveOperation, type_: &program::primitives::Type) -> InlineFunction {
    let primitive = primitive_from_primitive(type_);
    let primitive_u8 = primitive as u8;

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
        PrimitiveOperation::Negative => inline_fn_push_with_u8(OpCode::NEG, primitive_u8),
        PrimitiveOperation::Add => inline_fn_push_with_u8(OpCode::ADD, primitive_u8),
        PrimitiveOperation::Subtract => inline_fn_push_with_u8(OpCode::SUB, primitive_u8),
        PrimitiveOperation::Multiply => inline_fn_push_with_u8(OpCode::MUL, primitive_u8),
        PrimitiveOperation::Divide => inline_fn_push_with_u8(OpCode::DIV, primitive_u8),
        PrimitiveOperation::Modulo => inline_fn_push_with_u8(OpCode::MOD, primitive_u8),
        PrimitiveOperation::Exp => inline_fn_push_with_u8(OpCode::EXP, primitive_u8),
        PrimitiveOperation::Log => inline_fn_push_with_u8(OpCode::LOG, primitive_u8),
        PrimitiveOperation::EqualTo => inline_fn_push_with_u8(OpCode::EQ, primitive_u8),
        PrimitiveOperation::NotEqualTo => inline_fn_push_with_u8(OpCode::NEQ, primitive_u8),
        PrimitiveOperation::GreaterThan => inline_fn_push_with_u8(OpCode::GR, primitive_u8),
        PrimitiveOperation::LesserThan => inline_fn_push_with_u8(OpCode::LE, primitive_u8),
        PrimitiveOperation::GreaterThanOrEqual => inline_fn_push_with_u8(OpCode::GR_EQ, primitive_u8),
        PrimitiveOperation::LesserThanOrEqual => inline_fn_push_with_u8(OpCode::LE_EQ, primitive_u8),
        PrimitiveOperation::ParseIntString => inline_fn_push_with_u8(OpCode::PARSE, primitive_u8),
        PrimitiveOperation::ParseRealString => inline_fn_push_with_u8(OpCode::PARSE, primitive_u8),
        PrimitiveOperation::ToString => {
            let fun: IntrinsicFunction = match primitive {
                Primitive::U8 => |vm, sp| { unsafe { un_expr!(u8, ptr, to_str_ptr(val)); } },
                Primitive::U16 => |vm, sp| { unsafe { un_expr!(u16, ptr, to_str_ptr(val)); } },
                Primitive::U32 => |vm, sp| { unsafe { un_expr!(u32, ptr, to_str_ptr(val)); } },
                Primitive::U64 => |vm, sp| { unsafe { un_expr!(u64, ptr, to_str_ptr(val)); } },
                Primitive::I8 => |vm, sp| { unsafe { un_expr!(i8, ptr, to_str_ptr(val)); } },
                Primitive::I16 => |vm, sp| { unsafe { un_expr!(i16, ptr, to_str_ptr(val)); } },
                Primitive::I32 => |vm, sp| { unsafe { un_expr!(i32, ptr, to_str_ptr(val)); } },
                Primitive::I64 => |vm, sp| { unsafe { un_expr!(i64, ptr, to_str_ptr(val)); } },
                Primitive::F32 => |vm, sp| { unsafe { un_expr!(f32, ptr, to_str_ptr(val)); } },
                Primitive::F64 => |vm, sp| { unsafe { un_expr!(f64, ptr, to_str_ptr(val)); } },
                Primitive::BOOL => |vm, sp| { unsafe { un_expr!(bool, ptr, to_str_ptr(val)); } },
            };

            inline_fn_push_with_u64(CALL_INTRINSIC, unsafe { transmute(fun) })
        },
        PrimitiveOperation::Clone => inline_fn_push_identity(),  // Primitives are already pass-by-value.
    }
}