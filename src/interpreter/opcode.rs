#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    NOOP,
    PANIC,
    // TODO Replace with function call?
    TRANSPILE_ADD,
    // TODO Replace with function call?
    PRINT,
    LOAD0,
    LOAD8,
    LOAD16,
    LOAD32,
    LOAD64,
    LOAD_LOCAL_32,
    STORE_LOCAL_32,
    LOAD_CONSTANT_32,
    DUP64,
    POP64,
    SWAP64,
    JUMP,
    JUMP_IF_FALSE,
    AND,
    OR,
    NOT,
    NEG,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    EXP,
    LOG,
    EQ,
    NEQ,
    GR,
    GR_EQ,
    LE,
    LE_EQ,
    PARSE,
    TO_STRING,
    // TODO This can probably be done in-code some time (?)
    ADD_STRING,

    // Member
    ALLOC_32,
    SET_MEMBER_32,
    GET_MEMBER_32,

    CALL,
    RETURN,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Primitive {
    BOOL,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}
