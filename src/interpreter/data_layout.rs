use std::rc::Rc;

use itertools::Itertools;

use crate::program::allocation::ObjectReference;
use crate::program::traits::StructInfo;
use crate::program::types::TypeUnit;

pub struct DataLayout {
    pub struct_info: Rc<StructInfo>,
    pub fields: Vec<Rc<ObjectReference>>,
}

pub fn create_data_layout(struct_info: Rc<StructInfo>) -> Rc<DataLayout> {
    let mut fields = vec![];
    let mut todo = struct_info.fields.iter().rev().collect_vec();

    while let Some(next) = todo.pop() {
        match &next.type_.unit {
            TypeUnit::Void => unreachable!(),
            TypeUnit::Generic(_) => todo!(),
            TypeUnit::Struct(s) => {
                // TODO In the future, we probably want to merge nested structs into each
                //  other to avoid indirection. For now, it's safer and easier to just accept the
                //  indirection though.
                fields.push(Rc::clone(next));
            }
        }
    }

    Rc::new(DataLayout {
        struct_info,
        fields
    })
}
