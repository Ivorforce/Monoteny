pub mod builtins;
pub mod compiler;
pub mod run;
pub mod common;

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::rc::Rc;
use custom_error::custom_error;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use strum::IntoEnumIterator;
use crate::{linker, parser};
use crate::parser::ast;
use crate::program::allocation::ObjectReference;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, Statement};
use crate::program::functions::{FunctionHead, FunctionPointer, FunctionType};
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::module::Module;
use crate::program::traits::RequirementsFulfillment;


custom_error!{pub InterpreterError
    OSError{msg: String} = "OS Error: {msg}",
    ParserError{msg: String} = "Parser Error: {msg}",
    LinkerError{msg: String} = "Linker Error: {msg}",
    RuntimeError{msg: String} = "Runtime Error: {msg}",
}


pub type FunctionInterpreterImpl = Rc<dyn Fn(&mut FunctionInterpreter, ExpressionID, &RequirementsFulfillment) -> Option<Value>>;


pub struct Value {
    pub layout: Layout,
    pub data: *mut u8,
}

pub struct Runtime {
    pub builtins: Rc<Builtins>,

    // These are optimized for running and may not reflect the source code itself.
    // They are also only loaded on demand.
    pub function_evaluators: HashMap<Uuid, FunctionInterpreterImpl>,
    pub global_assignments: HashMap<Uuid, Value>,

    // These remain unchanged after linking.
    pub source: Source,
}

pub struct Source {
    pub module_by_name: HashMap<String, Box<Module>>,

    // Cache of aggregated module_by_name fields for quick reference.

    /// For each function_id, its head.
    pub fn_heads: HashMap<Uuid, Rc<FunctionHead>>,

    /// For each function, a usable reference to it as an object.
    pub fn_references: HashMap<Rc<FunctionHead>, Rc<ObjectReference>>,
    /// For each function, its 'default' representation for syntax.
    pub fn_pointers: HashMap<Rc<FunctionHead>, Rc<FunctionPointer>>,
    /// For relevant functions, their implementation.
    pub fn_implementations: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,
    /// For relevant functions, a hint what type of builtin it is.
    pub fn_builtin_hints: HashMap<Rc<FunctionHead>, BuiltinFunctionHint>,
}

pub struct FunctionInterpreter<'a> {
    pub runtime: &'a mut Runtime,
    pub implementation: Rc<FunctionImplementation>,
    pub requirements_fulfillment: Box<RequirementsFulfillment>,

    pub locals: HashMap<Uuid, Value>,
}

impl Runtime {
    pub fn new(builtins: &Rc<Builtins>) -> Box<Runtime> {
        let mut runtime = Box::new(Runtime {
            builtins: Rc::clone(builtins),
            function_evaluators: Default::default(),
            global_assignments: Default::default(),
            source: Source {
                module_by_name: Default::default(),
                fn_heads: Default::default(),
                fn_references: Default::default(),
                fn_pointers: Default::default(),
                fn_implementations: Default::default(),
                fn_builtin_hints: Default::default(),
            },
        });

        builtins::load(&mut runtime);
        for module in builtins.all_modules() {
            runtime.load_module(module);
        }

        runtime
    }

    pub fn load_file(&mut self, path: &PathBuf) -> Result<Box<Module>, InterpreterError> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| InterpreterError::OSError { msg: e.to_string() })?;
        self.load_source(&content)
    }

    pub fn load_source(&mut self, source: &str) -> Result<Box<Module>, InterpreterError> {
        let ast = parser::parse_program(source)
            .map_err(|e| InterpreterError::ParserError { msg: e.to_string() })?;
        self.load_ast(&ast)
    }

    pub fn load_ast(&mut self, syntax: &ast::Module) -> Result<Box<Module>, InterpreterError> {
        let mut scope = self.builtins.create_scope();

        for module in self.source.module_by_name.values() {
            scope.import(module)
                .map_err(|e| InterpreterError::LinkerError { msg: e.to_string() })?;
        }

        let module = linker::link_file(syntax, &scope, self)
            .map_err(|e| InterpreterError::LinkerError { msg: e.to_string() })?;

        self.load_module(&module);

        Ok(module)
    }

    pub fn load_module(&mut self, module: &Module) {
        self.source.fn_heads.extend(module.fn_pointers.keys().map(|f| (f.function_id, Rc::clone(f))).collect_vec());
        self.source.fn_references.extend(module.fn_references.clone());
        self.source.fn_pointers.extend(module.fn_pointers.clone());
        self.source.fn_implementations.extend(module.fn_implementations.clone());
        self.source.fn_builtin_hints.extend(module.fn_builtin_hints.clone());

        for (head, implementation) in module.fn_implementations.iter() {
            self.function_evaluators.insert(implementation.head.function_id.clone(), compiler::compile_function(implementation));

            unsafe {
                let fn_layout = Layout::new::<Uuid>();
                let ptr = alloc(fn_layout);
                *(ptr as *mut Uuid) = implementation.implementation_id;
                self.global_assignments.insert(
                    module.fn_references[head].id,
                    Value { data: ptr, layout: fn_layout }
                );
            }
        }
    }
}

impl FunctionInterpreter<'_> {
    pub unsafe fn assign_arguments(&mut self, arguments: Vec<Value>) {
        for (arg, parameter) in zip_eq(arguments, self.implementation.parameter_variables.iter()) {
            self.locals.insert(parameter.id.clone(), arg);
        }
    }

    pub unsafe fn run(&mut self) -> Option<Value> {
        // Avoid borrowing self.
        let self_implementation = Rc::clone(&self.implementation);
        for statement in self_implementation.statements.iter() {
            match statement.as_ref() {
                Statement::VariableAssignment(target, value) => {
                    let value = self.evaluate(*value);
                    self.locals.insert(target.id.clone(), value.unwrap());
                }
                Statement::Expression(expression_id) => {
                    self.evaluate(*expression_id);
                }
                Statement::Return(return_value) => {
                    return match return_value {
                        None => None,
                        Some(expression_id) => self.evaluate(*expression_id),
                    }
                }
            }
        }

        return None
    }

    pub fn combine_bindings(lhs: &RequirementsFulfillment, rhs: &RequirementsFulfillment) -> Box<RequirementsFulfillment> {
        todo!()
        // Box::new(TraitResolution {
        //     requirement_bindings: lhs.requirement_bindings.iter().chain(rhs.requirement_bindings.iter())
        //         .map(|(l, r)| (Rc::clone(l), r.clone()))
        //         .collect(),
        //     function_binding: todo!(),
        // })
    }

    pub fn resolve(&self, pointer: &FunctionHead) -> Uuid {
        match &pointer.function_type {
            FunctionType::Static => pointer.function_id.clone(),
            FunctionType::Polymorphic { provided_by_assumption, abstract_function } => {
                todo!();
                // if let Some(result) = self.resolution.requirement_bindings.get(requirement).and_then(|x| x.function_binding.get(abstract_function)) {
                //     return self.resolve(&result)
                // }

                panic!("Failed to resolve abstract function: {:?}", &pointer)
            },
        }
    }

    pub unsafe fn evaluate(&mut self, expression_id: ExpressionID) -> Option<Value> {
        // TODO We should probably create an interpretation tree and an actual VM, where abstract functions are statically pre-resolved.
        //  Function instances could be assigned an int ID and thus we can call the functions directly without a UUID hash lookup. Which should be nearly as fast as a switch statement.
        //  ExpressionOperation outer switch would be replaced by having a function for every call. Literals would be stored and copied somewhere else.
        //  FunctionInterpreter instances could also be cached - no need to re-create them recursively.
        //  This would be managed by a global interpreter that is expandable dynamically. i.e. it can be re-used for interactive environments and so on.
        // Avoid borrowing self.
        let self_implementation = Rc::clone(&self.implementation);
        match &self_implementation.expression_forest.operations[&expression_id] {
            ExpressionOperation::FunctionCall(call) => {
                let function_id = self.resolve(&call.function);

                guard!(let Some(implementation) = self.runtime.function_evaluators.get(&function_id) else {
                    panic!("Cannot find function ({}) with interface: {:?}", function_id, &call.function);
                });

                // Copy it to release the borrow on self.
                let implementation: FunctionInterpreterImpl = Rc::clone(&implementation);
                return implementation(self, expression_id, &call.requirements_fulfillment)
            }
            ExpressionOperation::PairwiseOperations { .. } => {
                panic!()
            }
            ExpressionOperation::VariableLookup(variable) => {
                return Some(
                    self.locals.get(&variable.id)
                        .or(self.runtime.global_assignments.get(&variable.id))
                        .expect(format!("Unknown Variable: {:?}", variable).as_str())
                        .clone()
                )
            }
            ExpressionOperation::ArrayLiteral => {
                panic!()
            }
            ExpressionOperation::StringLiteral(value) => {
                let string_layout = Layout::new::<String>();
                let ptr = alloc(string_layout);
                *(ptr as *mut String) = value.clone();
                return Some(Value { data: ptr, layout: string_layout })
            }
        }
    }

    pub unsafe fn evaluate_arguments(&mut self, expression_id: ExpressionID) -> Vec<Value> {
        // Avoid borrowing self.
        let self_implementation = Rc::clone(&self.implementation);
        self_implementation.expression_forest.arguments[&expression_id].iter()
            .map(|x| self.evaluate(*x).unwrap())
            .collect_vec()
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data, self.layout)
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = alloc(self.layout);
            std::ptr::copy_nonoverlapping(self.data, ptr, self.layout.size());
            return Value { data: ptr, layout: self.layout }
        }
    }
}