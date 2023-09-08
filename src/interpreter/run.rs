use std::alloc::{alloc, Layout};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::{builtins, compiler, FunctionInterpreter, FunctionInterpreterImpl, InterpreterGlobals, Value};
use crate::program::builtins::Builtins;
use crate::program::global::FunctionImplementation;
use crate::program::{find_annotated, Program};
use crate::program::traits::RequirementsFulfillment;
use crate::program::types::TypeUnit;


pub fn preload_program<'a>(program: &'a Program, evaluators: &mut HashMap<Uuid, FunctionInterpreterImpl<'a>>, assignments: &mut HashMap<Uuid, Value>) {
    for (function_pointer, implementation) in program.module.function_implementations.iter() {
        evaluators.insert(implementation.head.function_id.clone(), compiler::compile_function(implementation));

        unsafe {
            let fn_layout = Layout::new::<Uuid>();
            let ptr = alloc(fn_layout);
            *(ptr as *mut Uuid) = implementation.implementation_id;
            assignments.insert(
                program.module.functions_references[function_pointer].id,
                Value { data: ptr, layout: fn_layout }
            );
        }
    }
}


pub fn main(program: &Program, builtins: &Rc<Builtins>) {
    let entry_function = find_annotated(program.module.function_implementations.values(), "main").expect("No main function!");
    assert!(entry_function.head.interface.parameters.is_empty(), "@main function has parameters.");
    assert!(entry_function.head.interface.return_type.unit.is_void(), "@main function has a return value.");

    let mut evaluators = builtins::make_evaluators(builtins);
    let mut assignments = HashMap::new();

    preload_program(program, &mut evaluators, &mut assignments);

    let mut interpreter = FunctionInterpreter {
        globals: &mut InterpreterGlobals {
            builtins: Rc::clone(builtins),
            function_evaluators: evaluators,
        },
        implementation: Rc::clone(entry_function),
        // No parameters and return type = nothing to bind!
        requirements_fulfillment: RequirementsFulfillment::empty(),
        assignments,
    };
    unsafe {
        interpreter.run();
    }
}

pub fn transpile(program: &Program, builtins: &Rc<Builtins>, callback: &dyn Fn(&Rc<FunctionImplementation>)) {
    let entry_function = find_annotated(program.module.function_implementations.values(), "transpile").expect("No main function!");
    let mut evaluators = builtins::make_evaluators(builtins);
    let mut assignments = HashMap::new();

    preload_program(program, &mut evaluators, &mut assignments);

    let transpiler_obj = &entry_function.parameter_variables[0];

    // Set the transpiler object.
    unsafe {
        // We have nothing useful to set for now.
        // TODO In the future, we should differentiate between different transpiler objects.
        //  But that's certainly not needed for a while.
        let transpiler_layout = Layout::new::<Uuid>();
        let ptr = alloc(transpiler_layout);
        *(ptr as *mut Uuid) = Uuid::new_v4();
        assignments.insert(
            transpiler_obj.id,
            Value { data: ptr, layout: transpiler_layout }
        );
    }

    let mut implementations = HashMap::new();
    for implementation in program.module.function_implementations.values() {
        implementations.insert(implementation.implementation_id, Rc::clone(implementation));
    }

    let callback_cell = Rc::new(RefCell::new(callback));

    let b: FunctionInterpreterImpl = Rc::new(move |interpreter, expression_id, binding| {
        unsafe {
            let arguments = interpreter.evaluate_arguments(expression_id);
            let arg = &arguments[1];
            let arg_id = &interpreter.implementation.expression_forest.arguments[&expression_id][1];
            let arg_type = interpreter.implementation.type_forest.get_unit(arg_id).unwrap();

            // TODO Once we have a Function supertype we can remove this check.
            match arg_type {
                TypeUnit::Function(f) => {},
                _ => panic!("Argument to transpiler.add is not a function: {:?}", arg_type)
            };

            let implementation_id = *(arg.data as *const Uuid);
            let implementation = &implementations[&implementation_id];
            (&mut *callback_cell.borrow_mut())(implementation);

            return None;
        }
    });
    evaluators.insert(builtins.transpilation.add.target.function_id.clone(), b);

    let mut interpreter = FunctionInterpreter {
        globals: &mut InterpreterGlobals {
            builtins: Rc::clone(builtins),
            function_evaluators: evaluators,
        },
        implementation: Rc::clone(entry_function),
        // TODO Technically we should bind Transpiler here, probably to some internal transpiler we make up on the spot.
        requirements_fulfillment: RequirementsFulfillment::empty(),
        assignments,
    };
    unsafe {
        interpreter.run();
    }
}


