use std::rc::Rc;
use crate::interpreter::chunks::Chunk;
use crate::interpreter::data::Value;

pub struct CallFrame {
    pub chunk: Rc<Chunk>,
    pub ip: *const u8,
    pub fp: *mut Value,
}
