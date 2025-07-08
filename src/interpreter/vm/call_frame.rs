use crate::interpreter::chunks::Chunk;
use crate::interpreter::data::Value;
use std::rc::Rc;

pub struct CallFrame {
    pub chunk: Rc<Chunk>,
    pub ip: *const u8,
    pub fp: *mut Value,
}
