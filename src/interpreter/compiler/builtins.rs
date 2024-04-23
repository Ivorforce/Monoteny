use std::rc::Rc;
use crate::error::RResult;
use crate::interpreter::chunks::{Chunk, OpCode};
use crate::interpreter::compiler::get_function;
use crate::interpreter::Runtime;
use crate::program::calls::FunctionBinding;
use crate::program::global::FunctionLogicDescriptor;
use crate::program::module::{module_name, ModuleName};

pub fn compile_builtin_function_call(descriptor: &FunctionLogicDescriptor, binding: &Rc<FunctionBinding>, chunk: &mut Chunk, runtime: &Runtime) -> RResult<()> {
    // let core = runtime.get_or_load_module(&module_name("core"))?;

    // TODO This is the worst way to do this lol
    match descriptor {
        FunctionLogicDescriptor::Stub => {
            let Some(repr) = runtime.source.fn_representations.get(&binding.function) else {
                todo!();
            };

            match repr.name.as_str() {
                "add" => {
                    chunk.push(OpCode::TRANSPILE_ADD);
                },
                "_write_line" => {
                    chunk.push(OpCode::PRINT);
                }
                _ => todo!(),
            }
        },
        FunctionLogicDescriptor::TraitProvider(_) => todo!(),
        FunctionLogicDescriptor::FunctionProvider(fun) => get_function(fun, chunk),
        FunctionLogicDescriptor::PrimitiveOperation { .. } => todo!(),
        FunctionLogicDescriptor::Constructor(_) => todo!(),
        FunctionLogicDescriptor::GetMemberField(_, _) => todo!(),
        FunctionLogicDescriptor::SetMemberField(_, _) => todo!(),
    }

    Ok(())
}
