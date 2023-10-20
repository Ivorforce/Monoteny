use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use linked_hash_map::{Entry, LinkedHashMap};
use crate::source::Source;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::ExpressionOperation;
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionLogicDescriptor, FunctionImplementation};
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
        for expression_id in implementation.expression_tree.deep_children(implementation.expression_tree.root) {
            let operation = &implementation.expression_tree.values[&expression_id];

            if let ExpressionOperation::FunctionCall(binding) = operation {
                guard!(let Some(descriptor) = source.fn_logic_descriptors.get(&binding.function) else {
                    continue;
                });

                match descriptor {
                    FunctionLogicDescriptor::Constructor(trait_, fields) => {
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

                            for (head, descriptor) in source.fn_logic_descriptors.iter() {
                                match descriptor {
                                    FunctionLogicDescriptor::GetMemberField(trait_field, ref_) => {
                                        if trait_field == trait_ && fields.contains(ref_) {
                                            getters.insert(Rc::clone(ref_), Rc::clone(head));
                                        }
                                    }
                                    FunctionLogicDescriptor::SetMemberField(trait_field, ref_) => {
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
