use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;

use crate::{ast, parser, program, resolver};
use crate::error::{RResult, RuntimeError};
use crate::interpreter::builtins;
use crate::interpreter::chunks::Chunk;
use crate::interpreter::compiler::InlineFunction;
use crate::program::functions::FunctionHead;
use crate::program::module::{Module, module_name, ModuleName};
use crate::program::traits::Trait;
use crate::repository::Repository;
use crate::resolver::{imports, referencible, scopes};
use crate::source::Source;

pub struct Runtime {
    #[allow(non_snake_case)]
    pub Metatype: Rc<Trait>,
    pub primitives: Option<HashMap<program::primitives::Type, Rc<Trait>>>,
    pub traits: Option<program::builtins::traits::Traits>,

    // These are optimized for running and may not reflect the source code itself.
    // They are also only loaded on demand.
    pub function_evaluators: HashMap<Uuid, Chunk>,
    // TODO We'll need these only in the future when we compile functions to constants.
    // pub global_assignments: HashMap<Uuid, Value>,
    pub function_inlines: HashMap<Rc<FunctionHead>, InlineFunction>,

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
            function_evaluators: Default::default(),
            function_inlines: Default::default(),
            source: Source::new(),
            repository: Repository::new(),
        });

        let mut builtins_module = program::builtins::create_builtins(&mut runtime);
        referencible::add_trait(&mut runtime, &mut builtins_module, None, &Metatype).unwrap();

        runtime.source.module_by_name.insert(builtins_module.name.clone(), builtins_module);
        builtins::load(&mut runtime)?;

        Ok(runtime)
    }

    pub fn get_or_load_module(&mut self, name: &ModuleName) -> RResult<&Module> {
        // FIXME this should be if let Some( ... but the compiler bugs out
        if self.source.module_by_name.contains_key(name) {
            // Module is already loaded!
            return Ok(&self.source.module_by_name[name]);
        }

        // Gotta load the module first.
        let path = self.repository.resolve_module_path(name)?;
        let module = self.load_file_as_module(&path, name.clone())?;
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
        let mut scope = scopes::Scope::new();

        let builtins_name = module_name("builtins");
        if self.source.module_by_name.contains_key(&builtins_name) {
            imports::deep(self, builtins_name, &mut scope)?;
        }

        let core_name = module_name("core");
        if self.source.module_by_name.contains_key(&core_name) {
            imports::deep(self, core_name, &mut scope)?;
        }

        let mut module = Box::new(Module::new(name));
        resolver::resolve_file(syntax, &scope, self, &mut module)?;
        Ok(module)
    }
}
