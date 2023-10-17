pub mod builtins;
pub mod compiler;
pub mod run;

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::{linker, parser};
use crate::error::{RResult, RuntimeError};
use crate::interpreter::compiler::make_function_getter;
use crate::linker::{imports, scopes};
use crate::parser::ast;
use crate::program::builtins::Builtins;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::{FunctionHead, FunctionType};
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::module::{Module, module_name, ModuleName};
use crate::program::traits::{RequirementsFulfillment, Trait};
use crate::repository::Repository;


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
    // TODO We'll need these only in the future when we compile functions to constants.
    // pub global_assignments: HashMap<Uuid, Value>,

    // These remain unchanged after linking.
    pub source: Source,
    pub repository: Box<Repository>,
}

pub struct Source {
    pub module_by_name: HashMap<ModuleName, Box<Module>>,

    // Cache of aggregated module_by_name fields for quick reference.

    /// For each function_id, its head.
    pub trait_references: HashMap<Rc<FunctionHead>, Rc<Trait>>,

    /// For each function_id, its head.
    pub fn_heads: HashMap<Uuid, Rc<FunctionHead>>,
    /// For referencible functions, a way to load it. The getter itself does not get a getter.
    pub fn_getters: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
    /// For referencible functions, its 'default' representation for syntax.
    pub fn_representations: HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    /// For relevant functions, their implementation.
    pub fn_implementations: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,
    /// For relevant functions, a hint what type of core it is.
    pub fn_builtin_hints: HashMap<Rc<FunctionHead>, BuiltinFunctionHint>,
}

pub struct FunctionInterpreter<'a> {
    pub runtime: &'a mut Runtime,
    pub implementation: Rc<FunctionImplementation>,
    pub requirements_fulfillment: Rc<RequirementsFulfillment>,

    pub locals: HashMap<Uuid, Value>,
}

impl Runtime {
    pub fn new(builtins: &Rc<Builtins>) -> RResult<Box<Runtime>> {
        let mut runtime = Box::new(Runtime {
            builtins: Rc::clone(builtins),
            function_evaluators: Default::default(),
            source: Source {
                module_by_name: Default::default(),
                trait_references: Default::default(),
                fn_heads: Default::default(),
                fn_getters: Default::default(),
                fn_representations: Default::default(),
                fn_implementations: Default::default(),
                fn_builtin_hints: Default::default(),
            },
            repository: Repository::new(),
        });

        runtime.load(&builtins.module);
        builtins::load(&mut runtime)?;

        Ok(runtime)
    }

    pub fn get_or_load_module(&mut self, name: &ModuleName) -> RResult<&Module> {
        guard!(let Some(first_part) = name.first() else {
            return Err(RuntimeError::new(format!("{:?} is not a valid module name.", name)))
        });

        // FIXME this should be if let Some( ... but the compiler bugs out
        if self.source.module_by_name.contains_key(name) {
            // Module is already loaded!
            return Ok(&self.source.module_by_name[name]);
        }

        // Gotta load the module first.
        let path = self.repository.resolve_module_path(name)?;
        let module = self.load_file(&path, name.clone())?;
        self.source.module_by_name.insert(name.clone(), module);
        Ok(&self.source.module_by_name[name])
    }

    pub fn load_file(&mut self, path: &PathBuf, name: ModuleName) -> RResult<Box<Module>> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| RuntimeError::new(format!("Error loading {:?}: {}", path, e)))?;
        self.load_source(&content, name)
            .map_err(|errs| {
                errs.into_iter().map(|e| {
                    e.in_file(path.clone())
                }).collect_vec()
            })
    }

    pub fn load_source(&mut self, source: &str, name: ModuleName) -> RResult<Box<Module>> {
        // We can ignore the errors. All errors are stored inside the AST too and will fail there.
        // TODO When JIT loading is implemented, we should still try to link all non-loaded
        //  functions / modules and warn if they fail. We can also then warn they're unused too.
        let (ast, _) = parser::parse_program(source)?;
        self.load_ast(&ast, name)
    }

    pub fn load_ast(&mut self, syntax: &ast::Module, name: ModuleName) -> RResult<Box<Module>> {
        let mut scope = scopes::Scope::new();
        scope.import(&self.builtins.module).unwrap();

        let core_name = module_name("core");
        if self.source.module_by_name.contains_key(&core_name) {
            imports::deep(self, core_name, &mut scope)?;
        }

        let module = linker::link_file(syntax, &scope, self, name)?;

        self.load(&module);

        Ok(module)
    }

    pub fn load(&mut self, module: &Module) {
        self.source.trait_references.extend(module.trait_by_getter.clone());
        self.source.fn_heads.extend(module.fn_representations.keys().map(|f| (f.function_id, Rc::clone(f))).collect_vec());
        self.source.fn_getters.extend(module.fn_getters.clone());
        self.source.fn_representations.extend(module.fn_representations.clone());
        self.source.fn_implementations.extend(module.fn_implementations.clone());
        self.source.fn_builtin_hints.extend(module.fn_builtin_hints.clone());

        for (head, implementation) in module.fn_implementations.iter() {
            self.function_evaluators.insert(implementation.head.function_id, compiler::compile_function(implementation));

            // Function getter
            self.function_evaluators.insert(
                module.fn_getters[head].function_id,
                make_function_getter(implementation.implementation_id),  // FIXME you sure you don't need the head id?
            );
        }
    }
}

impl FunctionInterpreter<'_> {
    pub unsafe fn assign_arguments(&mut self, arguments: Vec<Value>) {
        for (arg, parameter) in zip_eq(arguments, self.implementation.parameter_locals.iter()) {
            self.locals.insert(parameter.id.clone(), arg);
        }
    }

    pub unsafe fn run(&mut self) -> Option<Value> {
        // Avoid borrowing self.
        self.evaluate(self.implementation.root_expression_id)
    }

    pub fn combine_bindings(lhs: &RequirementsFulfillment, rhs: &RequirementsFulfillment) -> Rc<RequirementsFulfillment> {
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
                    panic!("Interpreter cannot find function ({}) with interface: {:?}", function_id, &call.function);
                });

                // Copy it to release the borrow on self.
                let implementation: FunctionInterpreterImpl = Rc::clone(&implementation);
                implementation(self, expression_id, &call.requirements_fulfillment)
            }
            ExpressionOperation::PairwiseOperations { .. } => {
                panic!()
            }
            ExpressionOperation::GetLocal(variable) => {
                Some(
                    self.locals.get(&variable.id)
                        .expect(format!("Unknown Variable: {:?}", variable).as_str())
                        .clone()
                )
            }
            ExpressionOperation::SetLocal(target) => {
                let arguments = &self_implementation.expression_forest.arguments[&expression_id];
                assert_eq!(arguments.len(), 1);
                let new_value = self.evaluate(arguments[0]).unwrap();
                self.locals.insert(target.id.clone(), new_value);
                None
            }
            ExpressionOperation::ArrayLiteral => {
                panic!()
            }
            ExpressionOperation::StringLiteral(value) => {
                let string_layout = Layout::new::<String>();
                let ptr = alloc(string_layout);
                *(ptr as *mut String) = value.clone();
                Some(Value { data: ptr, layout: string_layout })
            }
            ExpressionOperation::Block => {
                let statements = &self_implementation.expression_forest.arguments[&expression_id];
                for statement in statements.iter() {
                    self.evaluate(*statement);
                }
                None  // Unusual, but a block might be just used inside a block, or a function that has no return value.
            }
            ExpressionOperation::Return => {
                let arguments = &self_implementation.expression_forest.arguments[&expression_id];

                // TODO Need a way to somehow bubble up to a 'named block'.
                match &arguments[..] {
                    [] => todo!(),
                    [arg] => {
                        let return_value = self.evaluate(*arg);
                        todo!()
                    },
                    _ => panic!()
                }
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