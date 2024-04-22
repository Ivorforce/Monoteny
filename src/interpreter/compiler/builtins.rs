use crate::error::RResult;
use crate::interpreter::chunks::Chunk;
use crate::program::global::FunctionLogicDescriptor;

pub fn compile_builtin_function(descriptor: &FunctionLogicDescriptor) -> RResult<Chunk> {
    todo!()
}
