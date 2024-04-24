#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    NOOP,
    RETURN,
    // TODO Replace with function call?
    TRANSPILE_ADD,
    // TODO Replace with function call?
    PRINT,
    LOAD8,
    LOAD16,
    LOAD32,
    LOAD64,
    LOAD128,
    LOAD_LOCAL,
    STORE_LOCAL,
    LOAD_CONSTANT,
    POP64,
    POP128,
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
