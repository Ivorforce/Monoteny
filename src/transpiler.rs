use std::collections::HashMap;
use std::rc::Rc;

use itertools::Itertools;

use crate::error::{RResult, TryCollectMany};
use crate::interpreter::runtime::Runtime;
use crate::program::functions::{FunctionHead, FunctionImplementation, FunctionLogic, FunctionLogicDescriptor};
use crate::refactor::simplify::Simplify;
use crate::refactor::Refactor;

pub mod python;
pub mod namespaces;
pub mod structs;
mod tests;

pub struct Config {
    pub should_constant_fold: bool,
    pub should_monomorphize: bool,
    pub should_inline: bool,
    pub should_trim_locals: bool,
}

impl Config {
    pub fn default() -> Config {
        Config {
            should_constant_fold: true,
            should_monomorphize: true,
            should_inline: true,
            should_trim_locals: true,
        }
    }
}

pub enum TranspiledArtifact {
    Function(Rc<FunctionHead>)
}

pub struct Transpiler {
    // In the future, this should all be accessible by monoteny code itself - including the context.
    pub main_function: Option<Rc<FunctionHead>>,
    pub exported_artifacts: Vec<TranspiledArtifact>,
}

pub struct TranspilePackage<'a> {
    // In the future, this should all be accessible by monoteny code itself - including the context.
    pub main_function: Option<Rc<FunctionHead>>,
    pub explicit_functions: Vec<(Rc<FunctionHead>, &'a FunctionImplementation)>,
    pub implicit_functions: Vec<(Rc<FunctionHead>, &'a FunctionImplementation)>,
    pub used_native_functions: HashMap<Rc<FunctionHead>, FunctionLogicDescriptor>,
}

pub trait LanguageContext {
    fn new(runtime: &Runtime) -> Self where Self: Sized;
    fn register_builtins(&self, refactor: &mut Refactor);
    fn refactor_code(&self, refactor: &mut Refactor);
    fn make_files(
        &self,
        base_filename: &str,
        package: TranspilePackage,
    ) -> RResult<HashMap<String, String>>;
}

pub fn transpile(transpiler: Box<Transpiler>, runtime: &mut Runtime, context: &dyn LanguageContext, config: &Config, base_filename: &str) -> RResult<HashMap<String, String>>{
    let mut refactor = Refactor::new();
    context.register_builtins(&mut refactor);

    let mut exported_function_order = vec![];

    for artifact in transpiler.exported_artifacts {
        match artifact {
            TranspiledArtifact::Function(function_head) => {
                match &runtime.source.fn_logic[&function_head] {
                    FunctionLogic::Implementation(implementation) => {
                        exported_function_order.push(Rc::clone(&function_head));
                        refactor.add(function_head, implementation.clone());
                    }
                    FunctionLogic::Descriptor(_) => panic!("Cannot transpile a function for which whe don't know an implementation!")
                }
            }
        }
    }

    if !config.should_monomorphize {
        todo!();  // Lots of reasons non-monomorphization doesn't work right now.
    }

    let mut simplify = Simplify {
        refactor,
        inline: config.should_inline,
        trim_locals: config.should_trim_locals,
        monomorphize: config.should_monomorphize,
    };

    simplify.run(exported_function_order.iter(), &runtime.source);

    // --- Reclaim from Refactor and make the ast
    context.refactor_code(&mut simplify.refactor);

    // TODO The call_graph doesn't know about calls made outside the refactor. If there was no monomorphization, some functions may not even be caught by this.
    let deep_calls = simplify.refactor.gather_deep_functions(exported_function_order.iter(), &runtime.source);
    let mut fn_logic = simplify.refactor.fn_logic;

    let exported_functions = simplify.refactor.explicit_functions.iter()
        .map(|head| Ok((Rc::clone(head), fn_logic.get(head).unwrap().as_implementation()?)))
        .try_collect_many()?;
    let mut implicit_functions = vec![];
    let mut native_functions = HashMap::new();

    for head in deep_calls {
        // Either Refactor has it (because it invented it) or it's unchanged from source.
        match fn_logic.get(&head).unwrap() {
            FunctionLogic::Implementation(i) => {
                implicit_functions.push((head, i.as_ref()));
            }
            FunctionLogic::Descriptor(d) => {
                native_functions.insert(head, d.clone());
            }
        }
    }

    context.make_files(base_filename, TranspilePackage {
        main_function: transpiler.main_function,
        explicit_functions: exported_functions,
        implicit_functions,
        used_native_functions: native_functions,
    })
}
