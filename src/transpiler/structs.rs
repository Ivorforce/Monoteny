use std::collections::HashMap;
use std::rc::Rc;

use linked_hash_map::{Entry, LinkedHashMap};

use crate::program::expression_tree::ExpressionOperation;
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogicDescriptor};
use crate::program::types::TypeProto;
use crate::source::StructInfo;

pub fn find_in_interfaces(heads: impl Iterator<Item=Rc<FunctionHead>>, map: &mut LinkedHashMap<Rc<TypeProto>, Rc<StructInfo>>) {
    for head in heads {
        for type_ in head.interface.parameters.iter().map(|p| &p.type_).chain([&head.interface.return_type].into_iter()) {
            todo!("From the type we SHOULD be able to deduce the struct info, but we can't for now.")
        }
    }
}

pub fn find_in_implementations(implementations: &Vec<&FunctionImplementation>, logic: &HashMap<Rc<FunctionHead>, FunctionLogicDescriptor>, map: &mut LinkedHashMap<Rc<TypeProto>, Rc<StructInfo>>) {
    for implementation in implementations {
        for expression_id in implementation.expression_tree.deep_children(implementation.expression_tree.root) {
            let operation = &implementation.expression_tree.values[&expression_id];

            if let ExpressionOperation::FunctionCall(binding) = operation {
                let Some(FunctionLogicDescriptor::Constructor(struct_info)) = logic.get(&binding.function) else {
                    continue;
                };

                let type_ = &binding.function.interface.return_type;  // Fulfillment for Self
                if let Entry::Vacant(entry) = map.entry(type_.clone()) {
                    // If it's already present and we insert, we shuffle it to the end, which is unnecessary
                    entry.insert(Rc::clone(struct_info));
                }
            }
        }
    }
}
