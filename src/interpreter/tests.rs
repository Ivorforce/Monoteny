#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::ptr::read_unaligned;

    use crate::error::RResult;
    use crate::interpreter;
    use crate::interpreter::chunks::Chunk;
    use crate::interpreter::compiler::compile_deep;
    use crate::interpreter::opcode::{OpCode, Primitive};
    use crate::interpreter::runtime::Runtime;
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
        chunk.push_with_u8(OpCode::ADD, Primitive::U16 as u8);
        // stack: [8]
        chunk.push_with_u16(OpCode::LOAD16, 4);
        chunk.push_with_u8(OpCode::DIV, Primitive::U16 as u8);
        // stack: [2]
        chunk.push_with_u16(OpCode::LOAD16, 2);
        chunk.push_with_u8(OpCode::EQ, Primitive::U16 as u8);
        // stack: [true]
        chunk.push(OpCode::RETURN);

        let mut out: Vec<u8> = vec![];
        let mut vm = VM::new(&chunk, &mut out);
        vm.run()?;

        unsafe {
            // There should be exactly one value left on the stack, at the very start, which is true
            let value = (*vm.stack.as_ptr()).bool;
            assert_eq!(value, true);
        }

        Ok(())
    }

    fn test_runs(path: &str) -> RResult<String> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_file_as_module(&PathBuf::from(path), module_name("main"))?;

        let entry_function = interpreter::run::get_main_function(&module)?.unwrap();

        // TODO Should gather all used functions and compile them
        let compiled = compile_deep(&mut runtime, entry_function)?;

        let mut out: Vec<u8> = vec![];
        let mut vm = VM::new(&compiled, &mut out);
        unsafe {
            vm.run()?;
        }

        Ok(std::str::from_utf8(&out).unwrap().to_string())
    }

    /// This tests the transpiler, interpreter and function calls.
    #[test]
    fn hello_world() -> RResult<()> {
        let out = test_runs("test-code/hello_world.monoteny")?;
        assert_eq!(out, "Hello World!\n");

        Ok(())
    }

    #[test]
    fn custom_grammar() -> RResult<()> {
        let out = test_runs("test-code/grammar/custom_grammar.monoteny")?;
        assert_eq!(out, "-3\n");

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
        let out = test_runs("test-code/grammar/string_interpolation.monoteny")?;
        assert_eq!(out, "Left: String, Right: 2\n");

        Ok(())
    }

    #[test]
    fn if_then_else() -> RResult<()> {
        let out = test_runs("test-code/control_flow/if_then_else.monoteny")?;
        assert_eq!(out, "true\n");

        Ok(())
    }

    #[test]
    fn and_or() -> RResult<()> {
        let out = test_runs("test-code/control_flow/and_or.monoteny")?;
        assert_eq!(out, "true and true\ntrue or true\nfalse or true\ntrue or false\n");

        Ok(())
    }

    #[test]
    fn traits_and_fields() -> RResult<()> {
        let out = test_runs("test-code/traits/simple.monoteny")?;
        assert_eq!(out, "Height 1: 180cm\nHeight 2: 150cm\n");

        Ok(())
    }

    // #[test]
    // fn assertions() -> RResult<()> {
    //     let mut runtime = Runtime::new()?;
    //     runtime.repository.add("common", PathBuf::from("monoteny"));
    //
    //     let module = runtime.load_file_as_module(&PathBuf::from("test-code/debug/assert.monoteny"), module_name("main"))?;
    //
    //     let entry_function = interpreter::run::get_main_function(&module)?.unwrap();
    //
    //     // TODO Should gather all used functions and compile them
    //     let compiled = compile_deep(&mut runtime, entry_function)?;
    //
    //     let mut out: Vec<u8> = vec![];
    //     let mut vm = VM::new(&compiled, &mut out);
    //     unsafe {
    //         match vm.run() {
    //             Ok(_) => assert!(false),
    //             Err(e) => {
    //                 assert_eq!(std::str::from_utf8(&out).unwrap().to_string(), "Assertion failure.");
    //             }
    //         }
    //     }
    //
    //     Ok(())
    // }
}
