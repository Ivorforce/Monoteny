use std::rc::Rc;
use itertools::Itertools;
use crate::interpreter::runtime::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::traits::StructInfo;
use crate::program::types::TypeUnit;

pub struct DataLayout {
    pub struct_info: Rc<StructInfo>,
    pub fields: Vec<Rc<ObjectReference>>,
}

pub fn create_data_layout(runtime: &mut Runtime, struct_info: Rc<StructInfo>) -> Rc<DataLayout> {
    let mut fields = vec![];
    let mut todo = struct_info.fields.iter().rev().collect_vec();

    while let Some(next) = todo.pop() {
        match &next.type_.unit {
            TypeUnit::Void => unreachable!(),
            TypeUnit::Generic(_) => todo!(),
            TypeUnit::Struct(s) => {
                if let Some(_) = runtime.primitives.as_ref().unwrap().iter().filter_map(|(p, t)| (t == s).then_some(p)).next() {
                    fields.push(Rc::clone(next));
                }
                else {
                    todo!("Need to 'embed' its data type into ours at this point")
                }
            }
        }
    }

    Rc::new(DataLayout {
        struct_info,
        fields
    })
}
