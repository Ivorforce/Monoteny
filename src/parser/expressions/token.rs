use crate::ast;
use crate::util::position::Positioned;

pub enum Value<'a, Function> {
    Operation(Function, Vec<Box<Positioned<Self>>>),
    Identifier(&'a String),
    MacroIdentifier(&'a String),
    RealLiteral(&'a String),
    IntLiteral(&'a String),
    StringLiteral(&'a Vec<Box<Positioned<ast::StringPart>>>),
    StructLiteral(&'a ast::Struct),
    ArrayLiteral(&'a ast::Array),
    Block(&'a ast::Block),
    MemberAccess(Box<Positioned<Self>>, &'a String),
    FunctionCall(Box<Positioned<Self>>, &'a ast::Struct),
    Subscript(Box<Positioned<Self>>, &'a ast::Array),
    IfThenElse(&'a ast::IfThenElse),
}

pub enum Token<'a, Function> {
    Keyword(Positioned<&'a String>),
    Value(Box<Positioned<Value<'a, Function>>>),
}
