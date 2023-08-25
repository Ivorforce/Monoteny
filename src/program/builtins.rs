use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use itertools::Itertools;
use crate::linker::scopes;
use crate::program::module;
use crate::program::module::Module;

pub mod precedence;
pub mod debug;
pub mod core;
pub mod primitives;
pub mod common;
pub mod transpilation;
pub mod traits;

pub struct Builtins {
    pub core: core::Core,
    pub precedence_groups: precedence::PrecedenceGroups,

    pub common: common::Common,

    pub debug: debug::Debug,
    pub transpilation: transpilation::Transpilation,

    pub module_by_name: HashMap<String, Rc<Module>>,
}

pub fn create_builtins() -> Rc<Builtins> {
    let core = core::create();
    let mut builtins = Builtins {
        common: common::create(&core),
        debug: debug::create(),
        transpilation: transpilation::create(&core),
        core,
        precedence_groups: precedence::create(),
        module_by_name: HashMap::new(),
    };

    builtins.module_by_name.insert(
        "math".into(),
        module::from_file(PathBuf::from("monoteny/common/math.monoteny"), &builtins).expect("Error compiling math library")
    );
    Rc::new(builtins)
}

impl Builtins {
    pub fn all_modules(&self) -> Vec<&Rc<Module>> {
        self.module_by_name.values().chain([
            &self.core.module,
            &self.precedence_groups.module,
            &self.common.module,
            &self.debug.module,
            &self.transpilation.module,
        ]).collect_vec()
    }

    pub fn create_scope(&self) -> scopes::Scope {
        let mut scope = scopes::Scope::new();

        for precedence_group in self.precedence_groups.list.iter() {
            scope.precedence_groups.push((Rc::clone(precedence_group), HashMap::new()));
        }

        for module in self.all_modules() {
            scope.import(module).unwrap();
        }

        scope
    }
}
