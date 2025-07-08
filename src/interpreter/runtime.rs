use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use itertools::Itertools;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::builtins;
use crate::interpreter::compile::compile_server::CompileServer;
use crate::interpreter::data::Value;
use crate::interpreter::vm::VM;
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionLogic, FunctionRepresentation};
use crate::program::module::{module_name, Module, ModuleName};
use crate::program::traits::Trait;
use crate::repository::Repository;
use crate::resolver::function::resolve_anonymous_expression;
use crate::resolver::{imports, referencible, scopes};
use crate::source::Source;
use crate::{ast, parser, program, repository, resolver};

pub struct Runtime {
    #[allow(non_snake_case)]
    pub Metatype: Rc<Trait>,
    pub primitives: Option<HashMap<program::primitives::Type, Rc<Trait>>>,
    pub traits: Option<builtins::traits::Traits>,

    pub base_scope: Rc<scopes::Scope<'static>>,
    pub compile_server: CompileServer,
    pub vm: VM,

    // These remain unchanged after resolution.
    pub source: Source,
    pub repository: Box<Repository>,
}

impl Runtime {
    #[allow(non_snake_case)]
    pub fn new() -> RResult<Box<Runtime>> {
        let mut Metatype = Trait::new_with_self("Type");
        let Metatype = Rc::new(Metatype);

        let mut runtime = Box::new(Runtime {
            Metatype: Rc::clone(&Metatype),
            primitives: None,
            traits: None,
            base_scope: Rc::new(scopes::Scope::new()),  // Temporary empty scope.
            compile_server: CompileServer::new(),
            vm: VM::new(),
            source: Source::new(),
            repository: Repository::new(),
        });

        let mut builtins_module = Box::new(Module::new(module_name("builtins")));

        runtime.primitives = Some(builtins::primitives::create_traits(&mut runtime, &mut builtins_module));
        runtime.traits = Some(builtins::traits::create(&mut runtime, &mut builtins_module));
        builtins::traits::create_functions(&mut runtime, &mut builtins_module);
        builtins::primitives::create_functions(&mut runtime, &mut builtins_module);

        referencible::add_trait(&mut runtime, &mut builtins_module, None, &Metatype).unwrap();

        // Load builtins
        runtime.source.module_by_name.insert(builtins_module.name.clone(), builtins_module);
        runtime.base_scope = Rc::new(runtime.make_scope()?);

        // Load core
        runtime.repository.add("core", builtins::modules::create_core_loader());
        runtime.get_or_load_module(&module_name("core"))?;

        // Final scope can be loaded.
        runtime.base_scope = Rc::new(runtime.make_scope()?);

        // Load VM builtins.
        builtins::vm::load(&mut runtime)?;

        Ok(runtime)
    }

    pub fn add_common_repository(&mut self) {
        self.repository.add("common", builtins::modules::create_common_loader());
    }

    fn make_scope(&mut self) -> RResult<scopes::Scope<'static>> {
        let mut scope = scopes::Scope::new();

        let builtins_name = module_name("builtins");
        if self.source.module_by_name.contains_key(&builtins_name) {
            imports::deep(self, builtins_name, &mut scope)?;
        }

        let core_name = module_name("core");
        if self.source.module_by_name.contains_key(&core_name) {
            imports::deep(self, core_name, &mut scope)?;
        }

        Ok(scope)
    }

    pub fn get_or_load_module(&mut self, name: &ModuleName) -> RResult<&Module> {
        // FIXME this should be if let Some( ... but the compiler bugs out
        if self.source.module_by_name.contains_key(name) {
            // Module is already loaded!
            return Ok(&self.source.module_by_name[name]);
        }

        // Gotta load the module first.
        let Some(first_part) = name.first() else {
            return Err(RuntimeError::error("Module name is empty...").to_array());
        };

        let Some(loader) = self.repository.entries.get(first_part) else {
            return Err(RuntimeError::error(format!("Module not in repository: {}", first_part).as_str()).to_array());
        };

        let module = match loader {
            repository::Loader::Path(base_path) => {
                let path = base_path.join(PathBuf::from(format!("{}.monoteny", name.join("/").as_str())));
                self.load_file_as_module(&path, name.clone())?
            },
            repository::Loader::Intrinsic(map) => {
                let text: &'static str = map.get(name)
                    .ok_or(RuntimeError::error(format!("Error loading {:?}: missing intrinsic", name).as_str()).to_array())?;
                self.load_text_as_module(&text, name.clone())
                    .map_err(|errs| {
                        errs.into_iter().map(|e| {
                            e.in_string(text)
                        }).collect_vec()
                    })?
            }
        };

        self.source.module_by_name.insert(name.clone(), module);
        Ok(&self.source.module_by_name[name])
    }

    pub fn load_file_as_module(&mut self, path: &PathBuf, name: ModuleName) -> RResult<Box<Module>> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| RuntimeError::error(format!("Error loading {:?}: {}", path, e).as_str()).to_array())?;
        self.load_text_as_module(&content, name)
            .map_err(|errs| {
                errs.into_iter().map(|e| {
                    e.in_file(path.clone())
                }).collect_vec()
            })
    }

    pub fn load_text_as_module(&mut self, source: &str, name: ModuleName) -> RResult<Box<Module>> {
        // We can ignore the errors. All errors are stored inside the AST too and will fail there.
        // TODO When JIT loading is implemented, we should still try to resolve all non-loaded
        //  functions / modules and warn if they fail. We can also then warn they're unused too.
        let (ast, _) = parser::parse_program(source)?;
        self.load_ast_as_module(&ast, name)
    }

    pub fn load_ast_as_module(&mut self, syntax: &ast::Block, name: ModuleName) -> RResult<Box<Module>> {
        let mut module = Box::new(Module::new(name));
        resolver::resolve_file(syntax, &Rc::clone(&self.base_scope), self, &mut module)?;
        Ok(module)
    }

    pub fn evaluate_anonymous_expression(&mut self, expression: &ast::Expression, interface: Rc<FunctionInterface>) -> RResult<Value> {
        // It doesn't make sense to evaluate something that isn't supposed to return anything.
        assert!(!interface.return_type.unit.is_void());

        let implementation = resolve_anonymous_expression(
            &interface, &expression, &Rc::clone(&self.base_scope), self
        )?;

        // TODO We shouldn't need a function head for this, I think.
        let dummy_head = FunctionHead::new_static(
            vec![],
            FunctionRepresentation::dummy(),
            interface,
        );
        self.source.fn_heads.insert(dummy_head.function_id, Rc::clone(&dummy_head));
        self.source.fn_logic.insert(Rc::clone(&dummy_head), FunctionLogic::Implementation(implementation));

        let compiled = self.compile_server.compile_deep(&self.source, &dummy_head)?;
        let result = self.vm.run(compiled, &self.compile_server, vec![])?;

        // We know by now that the expression is supposed to evaluate to something.
        return Ok(result.ok_or(RuntimeError::error("").to_array())?)
    }
}
