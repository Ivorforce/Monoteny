use crate::error::RResult;
use crate::interpreter::chunks::Chunk;
use crate::program::global::FunctionLogicDescriptor;

pub fn compile_builtin_function(descriptor: &FunctionLogicDescriptor, chunk: &mut Chunk) -> RResult<()> {
    todo!()
}
