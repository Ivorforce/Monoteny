#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::ptr::read_unaligned;

    use crate::error::RResult;
    use crate::interpreter;
    use crate::interpreter::chunks::Chunk;
    use crate::interpreter::opcode::{OpCode, Primitive};
    use crate::interpreter::Runtime;
    use crate::interpreter::vm::VM;
    use crate::program::module::{Module, module_name};
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

    fn test_runs(path: &str) -> RResult<Box<Module>> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_file_as_module(&PathBuf::from(path), module_name("main"))?;

        interpreter::run::main(&module, &mut runtime)?;

        Ok(module)
    }

    /// This tests the transpiler, interpreter and function calls.
    #[test]
    fn hello_world() -> RResult<()> {
        // TODO Pass a pipe and monitor that "Hello World!" is printed.
        let module = test_runs("test-code/hello_world.monoteny")?;
        assert_eq!(module.exposed_functions.len(), 2);

        Ok(())
    }

    #[test]
    fn custom_grammar() -> RResult<()> {
        test_runs("test-code/grammar/custom_grammar.monoteny")?;
        Ok(())
    }

    #[test]
    fn eq0() -> RResult<()> {
        test_runs("test-code/requirements/eq0.monoteny")?;
        Ok(())
    }

    #[test]
    fn eq1() -> RResult<()> {
        test_runs("test-code/requirements/eq1.monoteny")?;
        Ok(())
    }

    #[test]
    fn eq2() -> RResult<()> {
        test_runs("test-code/requirements/eq2.monoteny")?;
        Ok(())
    }

    #[test]
    fn string_interpolation() -> RResult<()> {
        test_runs("test-code/grammar/string_interpolation.monoteny")?;
        Ok(())
    }
}
