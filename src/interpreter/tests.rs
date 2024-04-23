#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::ExitCode;
    use std::ptr::read_unaligned;

    use crate::error::RResult;
    use crate::interpreter;
    use crate::interpreter::chunks::{Chunk, OpCode, Primitive};
    use crate::interpreter::Runtime;
    use crate::interpreter::vm::VM;
    use crate::program::module::module_name;
    use crate::transpiler::LanguageContext;

    /// This tests the transpiler, interpreter and function calls.
    #[test]
    fn run_expression() -> RResult<()> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let mut chunk = Chunk::new();
        chunk.push_with_u16(OpCode::LOAD16, 2);
        chunk.push_with_u16(OpCode::LOAD16, 6);
        chunk.push_with_u8(OpCode::ADD, Primitive::U32 as u8);
        chunk.push_with_u16(OpCode::LOAD16, 4);
        chunk.push_with_u8(OpCode::DIV, Primitive::U32 as u8);
        chunk.push_with_u16(OpCode::LOAD16, 2);
        chunk.push_with_u8(OpCode::EQ, Primitive::U32 as u8);
        chunk.push(OpCode::RETURN);

        let mut vm = VM::new(&chunk);
        vm.run()?;

        unsafe {
            let value = read_unaligned(vm.stack.as_ptr());
            assert_eq!(value.i64, 1);
        }

        Ok(())
    }

    // TODO Hello world is still broken.
    // /// This tests the transpiler, interpreter and function calls.
    // #[test]
    // fn run_hello_world() -> RResult<()> {
    //     let mut runtime = Runtime::new()?;
    //     runtime.repository.add("common", PathBuf::from("monoteny"));
    //
    //     let module = runtime.load_code(
    //         fs::read_to_string("test-code/hello_world.monoteny").unwrap().as_str(),
    //         module_name("main")
    //     )?;
    //
    //     assert_eq!(module.exposed_functions.len(), 2);
    //
    //     // TODO Pass a pipe and monitor that "Hello World!" is printed.
    //     interpreter::run::main(&module, &mut runtime)?;
    //
    //     Ok(())
    // }
}
