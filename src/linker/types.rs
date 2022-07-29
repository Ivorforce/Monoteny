use crate::abstract_syntax;
use crate::linker::computation_tree::Type;
use crate::linker::computation_tree::PrimitiveType;

pub fn resolve_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax {
        abstract_syntax::TypeDeclaration::Identifier(id) => {
            use PrimitiveType::*;
            match id.as_str() {
                "Bool" => Type::Primitive(Bool),
                "Int8" => Type::Primitive(Int8),
                "Int16" => Type::Primitive(Int16),
                "Int32" => Type::Primitive(Int32),
                "Int64" => Type::Primitive(Int64),
                "Int128" => Type::Primitive(Int128),
                "UInt8" => Type::Primitive(UInt8),
                "UInt16" => Type::Primitive(UInt16),
                "UInt32" => Type::Primitive(UInt32),
                "UInt64" => Type::Primitive(UInt64),
                "UInt128" => Type::Primitive(UInt128),
                "Float32" => Type::Primitive(Float32),
                "Float64" => Type::Primitive(Float64),
                _ => Type::Identifier(id.clone())
            }
        },
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(resolve_type(&identifier))
        }
    })
}
