use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;
use crate::program::builtins::traits;
use crate::program::builtins::traits::Traits;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::global::{BuiltinFunctionHint, PrimitiveOperation};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};

pub fn create_traits(module: &mut Module) -> HashMap<primitives::Type, Rc<Trait>> {
    let mut traits: HashMap<primitives::Type, Rc<Trait>> = Default::default();

    for primitive_type in primitives::Type::iter() {
        let trait_ = Rc::new(Trait::new(primitive_type.identifier_string()));
        module.add_trait(&trait_);
        traits.insert(primitive_type, trait_);
    }

    traits
}

pub fn create_functions(module: &mut Module, traits: &Traits, basis: &HashMap<primitives::Type, Rc<Trait>>) {
    let bool_type = TypeProto::simple_struct(&basis[&primitives::Type::Bool]);

    let mut add_function = |function: &Rc<FunctionPointer>, primitive_type: primitives::Type, operation: PrimitiveOperation, module: &mut Module| {
        module.add_function(&function);
        module.builtin_hints.insert(
            Rc::clone(&function),
            BuiltinFunctionHint::PrimitiveOperation { type_: primitive_type, operation }
        );
    };

    for (primitive_type, trait_) in basis.iter() {
        let type_ = TypeProto::simple_struct(&basis[primitive_type]);
        let primitive_type = *primitive_type;

        // Pair-Associative
        let eq_functions = traits::make_eq_functions(&type_, &bool_type);
        add_function(&eq_functions.equal_to, primitive_type, PrimitiveOperation::EqualTo, module);
        add_function(&eq_functions.not_equal_to, primitive_type, PrimitiveOperation::NotEqualTo, module);

        module.trait_conformance.add_conformance_manual(
            traits.Eq.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.Eq_functions.equal_to, &eq_functions.equal_to),
                (&traits.Eq_functions.not_equal_to, &eq_functions.not_equal_to),
            ]
        ).unwrap();

        if !primitive_type.is_number() {
            continue;
        }

        // Ord
        let ord_functions = traits::make_ord_functions(&type_, &bool_type);
        add_function(&ord_functions.greater_than, primitive_type, PrimitiveOperation::GreaterThan, module);
        add_function(&ord_functions.greater_than_or_equal_to, primitive_type, PrimitiveOperation::GreaterThanOrEqual, module);
        add_function(&ord_functions.lesser_than, primitive_type, PrimitiveOperation::LesserThan, module);
        add_function(&ord_functions.lesser_than_or_equal_to, primitive_type, PrimitiveOperation::LesserThanOrEqual, module);

        module.trait_conformance.add_conformance_manual(
            traits.Ord.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.Ord_functions.greater_than, &ord_functions.greater_than),
                (&traits.Ord_functions.greater_than_or_equal_to, &ord_functions.greater_than_or_equal_to),
                (&traits.Ord_functions.lesser_than, &ord_functions.lesser_than),
                (&traits.Ord_functions.lesser_than_or_equal_to, &ord_functions.lesser_than_or_equal_to),
            ]
        ).unwrap();

        // Number
        let number_functions = traits::make_number_functions(&type_, &bool_type);
        add_function(&number_functions.add, primitive_type, PrimitiveOperation::Add, module);
        add_function(&number_functions.subtract, primitive_type, PrimitiveOperation::Subtract, module);
        add_function(&number_functions.multiply, primitive_type, PrimitiveOperation::Multiply, module);
        add_function(&number_functions.divide, primitive_type, PrimitiveOperation::Divide, module);
        add_function(&number_functions.modulo, primitive_type, PrimitiveOperation::Modulo, module);
        add_function(&number_functions.negative, primitive_type, PrimitiveOperation::Negative, module);

        let _parse_int_literal = FunctionPointer::new_global(
            "parse_int_literal",
            FunctionInterface::new_operator(1, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&traits.String))), &type_)
        );
        add_function(&_parse_int_literal, primitive_type, PrimitiveOperation::ParseIntString, module);
        module.trait_conformance.add_conformance_manual(
            traits.ConstructableByIntLiteral.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.parse_int_literal_function, &_parse_int_literal),
            ]
        ).unwrap();

        module.trait_conformance.add_conformance_manual(
            traits.Number.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.Number_functions.add, &number_functions.add),
                (&traits.Number_functions.subtract, &number_functions.subtract),
                (&traits.Number_functions.multiply, &number_functions.multiply),
                (&traits.Number_functions.divide, &number_functions.divide),
                (&traits.Number_functions.modulo, &number_functions.modulo),
                // TODO This shouldn't exist for unsigned types
                (&traits.Number_functions.negative, &number_functions.negative),
            ]
        ).unwrap();

        if primitive_type.is_int() {
            module.trait_conformance.add_conformance_manual(traits.Int.create_generic_binding(vec![(&"self".into(), type_.clone())]), vec![]).unwrap();
        }

        if !(primitive_type.is_float()) {
            continue;
        }

        let float_functions = traits::make_float_functions(&type_);
        add_function(&float_functions.exponent, primitive_type, PrimitiveOperation::Exp, module);
        add_function(&float_functions.logarithm, primitive_type, PrimitiveOperation::Log, module);

        let _parse_float_literal = FunctionPointer::new_global(
            "parse_float_literal",
            FunctionInterface::new_operator(1, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&traits.String))), &type_)
        );
        add_function(&_parse_float_literal, primitive_type, PrimitiveOperation::ParseFloatString, module);
        module.trait_conformance.add_conformance_manual(
            traits.ConstructableByFloatLiteral.create_generic_binding(vec![(&"self".into(), type_.clone())]), vec![
                (&traits.parse_float_literal_function, &_parse_float_literal),
            ]
        ).unwrap();

        module.trait_conformance.add_conformance_manual(
            traits.Float.create_generic_binding(vec![(&"self".into(), type_)]), vec![
            (&traits.Float_functions.exponent, &float_functions.exponent),
            (&traits.Float_functions.logarithm, &float_functions.logarithm),
        ]).unwrap();
    }

    let and_op = FunctionPointer::new_global(
        "and_f",
        FunctionInterface::new_operator(2, &bool_type, &bool_type)
    );
    module.add_function(&and_op);
    module.builtin_hints.insert(
        Rc::clone(&and_op),
        BuiltinFunctionHint::PrimitiveOperation { type_: primitives::Type::Bool, operation: PrimitiveOperation::And }
    );

    let or__op = FunctionPointer::new_global(
        "or_f",
        FunctionInterface::new_operator(2, &bool_type, &bool_type)
    );
    module.add_function(&or__op);
    module.builtin_hints.insert(
        Rc::clone(&or__op),
        BuiltinFunctionHint::PrimitiveOperation { type_: primitives::Type::Bool, operation: PrimitiveOperation::Or }
    );

    let not_op = FunctionPointer::new_global(
        "not_f",
        FunctionInterface::new_operator(1, &bool_type, &bool_type)
    );
    module.add_function(&not_op);
    module.builtin_hints.insert(
        Rc::clone(&not_op),
        BuiltinFunctionHint::PrimitiveOperation { type_: primitives::Type::Bool, operation: PrimitiveOperation::Not }
    );
}
