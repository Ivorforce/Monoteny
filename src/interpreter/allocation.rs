use std::alloc::{alloc, dealloc, Layout};

pub struct Value {
    pub layout: Layout,
    pub data: *mut u8,
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data, self.layout)
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = alloc(self.layout);
            std::ptr::copy_nonoverlapping(self.data, ptr, self.layout.size());
            return Value { data: ptr, layout: self.layout }
        }
    }
}
