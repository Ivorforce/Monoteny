use crate::interpreter::data::Value;

#[inline(always)]
pub unsafe fn pop_stack(sp: &mut *mut Value) -> Value {
    *sp = (*sp).offset(-8);
    **sp
}
