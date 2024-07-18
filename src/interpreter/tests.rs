#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::rc::Rc;

    use uuid::Uuid;

    use crate::error::RResult;
    use crate::interpreter;
    use crate::interpreter::chunks::Chunk;
    use crate::interpreter::opcode::{OpCode, Primitive};
    use crate::interpreter::runtime::Runtime;
    use crate::interpreter::vm::VM;
    use crate::parser::parse_expression;
    use crate::program::expression_tree::ExpressionTree;
    use crate::program::functions::{FunctionHead, FunctionImplementation, FunctionInterface, FunctionLogic, FunctionRepresentation};
    use crate::program::generics::TypeForest;
    use crate::program::module::module_name;
    use crate::program::traits::RequirementsAssumption;
    use crate::program::types::TypeProto;
    use crate::resolver::imperative::ImperativeResolver;
    use crate::resolver::imperative_builder::ImperativeBuilder;
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
        let mut vm = VM::new(&mut out);
        let result = vm.run(Rc::new(chunk), &mut runtime, vec![])?;

        unsafe { assert_eq!(result.unwrap().bool, true); }

        Ok(())
    }

    fn test_runs(path: &str) -> RResult<String> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_file_as_module(&PathBuf::from(path), module_name("main"))?;

        let entry_function = interpreter::run::get_main_function(&module)?.unwrap();

        // TODO Should gather all used functions and compile them
        let compiled = runtime.compile_server.compile_deep(&runtime.source, entry_function)?;

        let mut out: Vec<u8> = vec![];
        let mut vm = VM::new(&mut out);
        unsafe {
            vm.run(compiled, &mut runtime, vec![])?;
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
    fn monomorphization_branch() -> RResult<()> {
        let out = test_runs("test-code/monomorphization/branch.monoteny")?;
        assert_eq!(out, "9\n9\n");

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
    fn traits_one_field() -> RResult<()> {
        let out = test_runs("test-code/traits/simple.monoteny")?;
        assert_eq!(out, "Height 1: 180cm\nHeight 2: 150cm\n");

        Ok(())
    }

    #[test]
    fn traits_fields() -> RResult<()> {
        let out = test_runs("test-code/traits/fields.monoteny")?;
        assert_eq!(out, "Noir (Cat) was: 180cm\nAnd is now: 25cm\n");

        Ok(())
    }

    #[test]
    fn assertions() -> RResult<()> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_file_as_module(&PathBuf::from("test-code/debug/assert.monoteny"), module_name("main"))?;

        let entry_function = interpreter::run::get_main_function(&module)?.unwrap();

        // TODO Should gather all used functions and compile them
        let compiled = runtime.compile_server.compile_deep(&runtime.source, entry_function)?;

        let mut out: Vec<u8> = vec![];
        let mut vm = VM::new(&mut out);
        unsafe {
            let result = vm.run(compiled, &mut runtime, vec![]);
            assert_eq!(std::str::from_utf8(&out).unwrap().to_string(), "Assertion failure.\n");

            if let Ok(_) = result {
                assert!(false)
            }
        }

        Ok(())
    }

    #[test]
    fn anonymous_type() -> RResult<()> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let (parsed_source, _) = parse_expression("String")?;

        let function_interface = FunctionInterface::new_provider(
            &TypeProto::one_arg(&runtime.Metatype, TypeProto::unit_struct(&runtime.traits.as_ref().unwrap().String)),
            vec![]
        );

        let mut scope = runtime.make_scope()?;

        let mut builder = ImperativeBuilder {
            runtime: &runtime,
            types: Box::new(TypeForest::new()),
            expression_tree: Box::new(ExpressionTree::new(Uuid::new_v4())),
            locals_names: Default::default(),
        };

        let mut resolver = ImperativeResolver {
            return_type: Rc::clone(&function_interface.return_type),
            builder,
            ambiguities: vec![],
        };

        let head_expression = resolver.resolve_expression(&parsed_source, &scope)?;
        resolver.builder.types.bind(head_expression, &function_interface.return_type)?;
        resolver.builder.expression_tree.root = head_expression;  // TODO This is kinda dumb; but we can't write into an existing head expression
        resolver.resolve_all_ambiguities()?;

        let implementation = Box::new(FunctionImplementation {
            interface: Rc::clone(&function_interface),
            requirements_assumption: RequirementsAssumption::empty(),
            expression_tree: resolver.builder.expression_tree,
            type_forest: resolver.builder.types,
            parameter_locals: vec![],
            locals_names: resolver.builder.locals_names,
        });

        // TODO We shouldn't need a function head for this.
        let dummy_head = FunctionHead::new_static(
            vec![],
            FunctionRepresentation::dummy(),
            function_interface,
        );
        runtime.source.fn_heads.insert(dummy_head.function_id, Rc::clone(&dummy_head));
        runtime.source.fn_logic.insert(Rc::clone(&dummy_head), FunctionLogic::Implementation(implementation));

        let compiled = runtime.compile_server.compile_deep(&runtime.source, &dummy_head)?;

        let mut out: Vec<u8> = vec![];
        let mut vm = VM::new(&mut out);
        let result = vm.run(compiled, &mut runtime, vec![])?;

        unsafe {
            let uuid = *(result.unwrap().ptr as *mut Uuid);
            assert_eq!(uuid, runtime.traits.unwrap().String.id);
        }

        Ok(())
    }
}
