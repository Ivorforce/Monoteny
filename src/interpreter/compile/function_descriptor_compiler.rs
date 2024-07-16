use std::rc::Rc;

use crate::interpreter::compile::compile_server::CompileServer;
use crate::interpreter::data::{uuid_to_ptr, Value};
use crate::interpreter::opcode::OpCode;
use crate::program::functions::{FunctionHead, FunctionLogicDescriptor};

pub fn compile_descriptor(function: &Rc<FunctionHead>, descriptor: &FunctionLogicDescriptor, compile_server: &mut CompileServer) {
    match descriptor {
        FunctionLogicDescriptor::Stub => todo!("{:?}", function),
        FunctionLogicDescriptor::Clone(_) => todo!("{:?}", function),
        FunctionLogicDescriptor::TraitProvider(trait_) => {
            let uuid = trait_.id;
            compile_server.function_inlines.insert(Rc::clone(function), Rc::new(move |compiler, expression| {
                unsafe { compiler.chunk.constants.push(Value { ptr: uuid_to_ptr(uuid) }); }
                compiler.chunk.push_with_u32(OpCode::LOAD_CONSTANT_32, u32::try_from(compiler.chunk.constants.len() - 1).unwrap());
                Ok(())
            }));
        }
        FunctionLogicDescriptor::FunctionProvider(f) => {
            let uuid = f.function_id;
            compile_server.function_inlines.insert(Rc::clone(function), Rc::new(move |compiler, expression| {
                unsafe { compiler.chunk.constants.push(Value { ptr: uuid_to_ptr(uuid) }); }
                compiler.chunk.push_with_u32(OpCode::LOAD_CONSTANT_32, u32::try_from(compiler.chunk.constants.len() - 1).unwrap());
                Ok(())
            }));
        }
        FunctionLogicDescriptor::PrimitiveOperation { .. } => todo!("{:?}", descriptor),
        FunctionLogicDescriptor::Constructor(struct_info) => {
            let data_layout = compile_server.get_data_layout(struct_info);

            compile_server.function_inlines.insert(Rc::clone(function), Rc::new(move |compiler, expression| {
                let arguments = &compiler.implementation.expression_tree.children[&expression];
                assert_eq!(arguments.len(), data_layout.fields.len() + 1);

                compiler.chunk.push_with_u32(OpCode::ALLOC_32, u32::try_from(data_layout.fields.len()).unwrap());
                for (idx, arg) in arguments.iter().skip(1).enumerate() {
                    // If needed, duplicate the object pointer.
                    if idx < arguments.len() - 1 {
                        compiler.chunk.push(OpCode::DUP64);
                    }

                    // Evaluate the field at the given index.
                    compiler.compile_expression(arg)?;
                    compiler.chunk.push_with_u32(OpCode::SET_MEMBER_32, u32::try_from(idx).unwrap());
                }

                Ok(())
            }));
        }
        FunctionLogicDescriptor::GetMemberField(struct_info, ref_) => {
            let data_layout = compile_server.get_data_layout(struct_info);
            let slot_index = data_layout.fields.iter().position(|r| r == ref_).unwrap();

            compile_server.function_inlines.insert(Rc::clone(function), Rc::new(move |compiler, expression| {
                let arguments = &compiler.implementation.expression_tree.children[&expression];

                compiler.compile_expression(&arguments[0])?;

                compiler.chunk.push_with_u32(OpCode::GET_MEMBER_32, u32::try_from(slot_index).unwrap());

                Ok(())
            }));
        }
        FunctionLogicDescriptor::SetMemberField(struct_info, ref_) => {
            let data_layout = compile_server.get_data_layout(struct_info);
            let slot_index = data_layout.fields.iter().position(|r| r == ref_).unwrap();

            compile_server.function_inlines.insert(Rc::clone(function), Rc::new(move |compiler, expression| {
                let arguments = &compiler.implementation.expression_tree.children[&expression];

                compiler.compile_expression(&arguments[0])?;
                compiler.compile_expression(&arguments[1])?;

                compiler.chunk.push_with_u32(OpCode::SET_MEMBER_32, u32::try_from(slot_index).unwrap());

                Ok(())
            }));
        }
    }
}
