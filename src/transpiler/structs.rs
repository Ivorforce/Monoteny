use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use linked_hash_map::{Entry, LinkedHashMap};
use crate::interpreter::Source;
use crate::linker::interface::FunctionHead;
use crate::program::allocation::ObjectReference;
use crate::program::computation_tree::ExpressionOperation;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::traits::Trait;
use crate::program::types::TypeProto;

pub struct Struct {
    pub type_: Box<TypeProto>,
    pub trait_: Rc<Trait>,

    pub constructor: Rc<FunctionHead>,
    pub fields: Vec<Rc<ObjectReference>>,
    pub getters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
    pub setters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
}

pub fn find(implementations: &Vec<Box<FunctionImplementation>>, source: &Source, map: &mut LinkedHashMap<Box<TypeProto>, Struct>) {
    for implementation in implementations {
        for expression_id in implementation.expression_forest.deep_children(implementation.root_expression_id) {
            let operation = &implementation.expression_forest.operations[&expression_id];

            if let ExpressionOperation::FunctionCall(binding) = operation {
                guard!(let Some(hint) = source.fn_builtin_hints.get(&binding.function) else {
                    continue;
                });
                let hint: &BuiltinFunctionHint = hint;

                match hint {
                    BuiltinFunctionHint::Constructor(trait_, fields) => {
                        let type_ = &binding.function.interface.return_type;  // Fulfillment for Self
                        if let Entry::Vacant(entry) = map.entry(type_.clone()) {
                            // TODO If we have generics, we should include their bindings in the name somehow.
                            //  Eg. ArrayFloat. Probably only if it's exactly one. Otherwise, we need to be ok with
                            //  just the auto-renames.

                            // TODO This logic will fall apart if we have multiple instantiations of the same type.
                            //  In that case we probably want to monomorphize the struct getter per-object so we can
                            //  differentiate them and assign different names.

                            let mut getters = HashMap::new();
                            let mut setters = HashMap::new();

                            for (head, hint) in source.fn_builtin_hints.iter() {
                                match hint {
                                    BuiltinFunctionHint::GetMemberField(trait_field, ref_) => {
                                        if trait_field == trait_ && fields.contains(ref_) {
                                            getters.insert(Rc::clone(ref_), Rc::clone(head));
                                        }
                                    }
                                    BuiltinFunctionHint::SetMemberField(trait_field, ref_) => {
                                        if trait_field == trait_ && fields.contains(ref_) {
                                            setters.insert(Rc::clone(ref_), Rc::clone(head));
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            entry.insert(Struct {
                                type_: type_.clone(),
                                trait_: Rc::clone(trait_),
                                constructor: Rc::clone(&binding.function),
                                fields: fields.clone(),
                                getters,
                                setters,
                            });
                        }
                    }
                    _ => {},
                }
            }
        }
    }
}
