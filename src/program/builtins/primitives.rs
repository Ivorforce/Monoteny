use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;
use crate::program::builtins::traits;
use crate::program::builtins::traits::{make_to_string_function, Traits};
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
        module.fn_builtin_hints.insert(
            Rc::clone(&function.target),
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
                (&traits.Eq_functions.equal_to.target, &eq_functions.equal_to.target),
                (&traits.Eq_functions.not_equal_to.target, &eq_functions.not_equal_to.target),
            ]
        ).unwrap();

        let to_string_function = make_to_string_function(&traits.ToString, &traits.String);
        add_function(&to_string_function, primitive_type, PrimitiveOperation::ToString, module);
        module.trait_conformance.add_conformance_manual(
            traits.ToString.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.to_string_function.target, &to_string_function.target),
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
                (&traits.Ord_functions.greater_than.target, &ord_functions.greater_than.target),
                (&traits.Ord_functions.greater_than_or_equal_to.target, &ord_functions.greater_than_or_equal_to.target),
                (&traits.Ord_functions.lesser_than.target, &ord_functions.lesser_than.target),
                (&traits.Ord_functions.lesser_than_or_equal_to.target, &ord_functions.lesser_than_or_equal_to.target),
            ]
        ).unwrap();

        // Number
        let number_functions = traits::make_number_functions(&type_);
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
                (&traits.parse_int_literal_function.target, &_parse_int_literal.target),
            ]
        ).unwrap();

        module.trait_conformance.add_conformance_manual(
            traits.Number.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.Number_functions.add.target, &number_functions.add.target),
                (&traits.Number_functions.subtract.target, &number_functions.subtract.target),
                (&traits.Number_functions.multiply.target, &number_functions.multiply.target),
                (&traits.Number_functions.divide.target, &number_functions.divide.target),
                (&traits.Number_functions.modulo.target, &number_functions.modulo.target),
                // TODO This shouldn't exist for unsigned types
                (&traits.Number_functions.negative.target, &number_functions.negative.target),
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
                (&traits.parse_float_literal_function.target, &_parse_float_literal.target),
            ]
        ).unwrap();

        module.trait_conformance.add_conformance_manual(
            traits.Float.create_generic_binding(vec![(&"self".into(), type_)]), vec![
            (&traits.Float_functions.exponent.target, &float_functions.exponent.target),
            (&traits.Float_functions.logarithm.target, &float_functions.logarithm.target),
        ]).unwrap();
    }

    let and_op = FunctionPointer::new_global(
        "and_f",
        FunctionInterface::new_operator(2, &bool_type, &bool_type)
    );
    module.add_function(&and_op);
    module.fn_builtin_hints.insert(
        Rc::clone(&and_op.target),
        BuiltinFunctionHint::PrimitiveOperation { type_: primitives::Type::Bool, operation: PrimitiveOperation::And }
    );

    let or__op = FunctionPointer::new_global(
        "or_f",
        FunctionInterface::new_operator(2, &bool_type, &bool_type)
    );
    module.add_function(&or__op);
    module.fn_builtin_hints.insert(
        Rc::clone(&or__op.target),
        BuiltinFunctionHint::PrimitiveOperation { type_: primitives::Type::Bool, operation: PrimitiveOperation::Or }
    );

    let not_op = FunctionPointer::new_global(
        "not_f",
        FunctionInterface::new_operator(1, &bool_type, &bool_type)
    );
    module.add_function(&not_op);
    module.fn_builtin_hints.insert(
        Rc::clone(&not_op.target),
        BuiltinFunctionHint::PrimitiveOperation { type_: primitives::Type::Bool, operation: PrimitiveOperation::Not }
    );
}
