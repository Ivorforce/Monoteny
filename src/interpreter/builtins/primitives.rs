use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::builtins::traits;
use crate::interpreter::builtins::traits::make_to_string_function;
use crate::interpreter::runtime::Runtime;
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionLogic, FunctionLogicDescriptor, FunctionRepresentation, PrimitiveOperation};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitConformanceRule};
use crate::program::types::{TypeProto, TypeUnit};
use crate::resolver::referencible;

pub fn create_traits(runtime: &mut Runtime, module: &mut Module) -> HashMap<primitives::Type, Rc<Trait>> {
    let mut traits: HashMap<primitives::Type, Rc<Trait>> = Default::default();

    for primitive_type in [
        primitives::Type::Bool,
        primitives::Type::Int(8),
        primitives::Type::Int(16),
        primitives::Type::Int(32),
        primitives::Type::Int(64),
        primitives::Type::UInt(8),
        primitives::Type::UInt(16),
        primitives::Type::UInt(32),
        primitives::Type::UInt(64),
        primitives::Type::Float(32),
        primitives::Type::Float(64),
    ] {
        let trait_ = Rc::new(Trait::new_with_self(&primitive_type.identifier_string()));
        referencible::add_trait(runtime, module, None, &trait_).unwrap();
        traits.insert(primitive_type, trait_);
    }

    traits
}

#[allow(non_snake_case)]
pub fn create_functions(runtime: &mut Runtime, module: &mut Module) {
    // TODO Cloning is dumb but we can't hold a runtime reference.
    //  It's not too bad because it's all Rcs though.
    let traits = runtime.traits.as_ref().unwrap().clone();
    let primitive_traits = runtime.primitives.as_ref().unwrap().clone();
    let bool_type = TypeProto::unit_struct(&primitive_traits[&primitives::Type::Bool]);

    let mut add_function = |function: &Rc<FunctionHead>, primitive_type: primitives::Type, operation: PrimitiveOperation, module: &mut Module, runtime: &mut Runtime| {
        referencible::add_function(runtime, module, None, function).unwrap();
        runtime.source.fn_logic.insert(
            Rc::clone(&function),
            FunctionLogic::Descriptor(FunctionLogicDescriptor::PrimitiveOperation { type_: primitive_type, operation })
        );
    };

    for (primitive_type, trait_) in primitive_traits.iter() {
        let type_ = TypeProto::unit_struct(&primitive_traits[primitive_type]);
        let primitive_type = *primitive_type;

        // Any!
        let any_functions = traits::make_any_functions(&type_);
        add_function(&any_functions.clone, primitive_type, PrimitiveOperation::Clone, module, runtime);
        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.Any.create_generic_binding(vec![("Self", type_.clone())]),
            vec![
                (&traits.Any_functions.clone, &any_functions.clone),
            ]
        ));

        // Pair-Associative
        let eq_functions = traits::make_eq_functions(&type_, &bool_type);
        add_function(&eq_functions.equal_to, primitive_type, PrimitiveOperation::EqualTo, module, runtime);
        add_function(&eq_functions.not_equal_to, primitive_type, PrimitiveOperation::NotEqualTo, module, runtime);

        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.Eq.create_generic_binding(vec![("Self", type_.clone())]),
            vec![
                (&traits.Eq_functions.equal_to, &eq_functions.equal_to),
                (&traits.Eq_functions.not_equal_to, &eq_functions.not_equal_to),
            ]
        ));

        let to_string_function = make_to_string_function(&traits.ToString, &traits.String);
        add_function(&to_string_function, primitive_type, PrimitiveOperation::ToString, module, runtime);
        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.ToString.create_generic_binding(vec![("Self", type_.clone())]),
            vec![
                (&traits.to_string_function, &to_string_function),
            ]
        ));

        if !primitive_type.is_number() {
            continue;
        }

        // Ord
        let ord_functions = traits::make_ord_functions(&type_, &bool_type);
        add_function(&ord_functions.greater_than, primitive_type, PrimitiveOperation::GreaterThan, module, runtime);
        add_function(&ord_functions.greater_than_or_equal_to, primitive_type, PrimitiveOperation::GreaterThanOrEqual, module, runtime);
        add_function(&ord_functions.lesser_than, primitive_type, PrimitiveOperation::LesserThan, module, runtime);
        add_function(&ord_functions.lesser_than_or_equal_to, primitive_type, PrimitiveOperation::LesserThanOrEqual, module, runtime);

        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.Ord.create_generic_binding(vec![("Self", type_.clone())]),
            vec![
                (&traits.Ord_functions.greater_than, &ord_functions.greater_than),
                (&traits.Ord_functions.greater_than_or_equal_to, &ord_functions.greater_than_or_equal_to),
                (&traits.Ord_functions.lesser_than, &ord_functions.lesser_than),
                (&traits.Ord_functions.lesser_than_or_equal_to, &ord_functions.lesser_than_or_equal_to),
            ]
        ));

        // Number
        let number_functions = traits::make_number_functions(&type_);
        add_function(&number_functions.add, primitive_type, PrimitiveOperation::Add, module, runtime);
        add_function(&number_functions.subtract, primitive_type, PrimitiveOperation::Subtract, module, runtime);
        add_function(&number_functions.multiply, primitive_type, PrimitiveOperation::Multiply, module, runtime);
        add_function(&number_functions.divide, primitive_type, PrimitiveOperation::Divide, module, runtime);
        add_function(&number_functions.modulo, primitive_type, PrimitiveOperation::Modulo, module, runtime);
        add_function(&number_functions.negative, primitive_type, PrimitiveOperation::Negative, module, runtime);

        let _parse_int_literal = FunctionHead::new_static(
            FunctionHead::dummy_param_names(1),
            FunctionRepresentation::new_global_function("parse_int_literal"),
            FunctionInterface::new_operator(1, &TypeProto::unit_struct(&traits.String), &type_),
        );
        add_function(&_parse_int_literal, primitive_type, PrimitiveOperation::ParseIntString, module, runtime);
        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.ConstructableByIntLiteral.create_generic_binding(vec![("Self", type_.clone())]),
            vec![
                (&traits.parse_int_literal_function, &_parse_int_literal),
            ]
        ));

        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.Number.create_generic_binding(vec![("Self", type_.clone())]),
            vec![
                (&traits.Number_functions.add, &number_functions.add),
                (&traits.Number_functions.subtract, &number_functions.subtract),
                (&traits.Number_functions.multiply, &number_functions.multiply),
                (&traits.Number_functions.divide, &number_functions.divide),
                (&traits.Number_functions.modulo, &number_functions.modulo),
                (&traits.Number_functions.negative, &number_functions.negative),
            ]
        ));

        if primitive_type.is_int() {
            module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
                traits.Int.create_generic_binding(vec![("Self", type_.clone())]),
                vec![]
            ));

            if !primitive_type.is_signed_number() {
                module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
                    traits.Natural.create_generic_binding(vec![("Self", type_.clone())]),
                    vec![]
                ));
            }
        }

        if !(primitive_type.is_float()) {
            continue;
        }

        let real_functions = traits::make_real_functions(&type_);
        add_function(&real_functions.pow, primitive_type, PrimitiveOperation::Exp, module, runtime);
        add_function(&real_functions.log, primitive_type, PrimitiveOperation::Log, module, runtime);

        let _parse_real_literal = FunctionHead::new_static(
            FunctionHead::dummy_param_names(1),
            FunctionRepresentation::new_global_function("parse_real_literal"),
            FunctionInterface::new_operator(1, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&traits.String))), &type_),
        );
        add_function(&_parse_real_literal, primitive_type, PrimitiveOperation::ParseRealString, module, runtime);
        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.ConstructableByRealLiteral.create_generic_binding(vec![("Self", type_.clone())]),
            vec![
                (&traits.parse_real_literal_function, &_parse_real_literal),
            ]
        ));

        module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
            traits.Real.create_generic_binding(vec![("Self", type_)]),
            vec![
                (&traits.Real_functions.pow, &real_functions.pow),
                (&traits.Real_functions.log, &real_functions.log),
            ]
        ));
    }

    let and_op = FunctionHead::new_static(
        FunctionHead::dummy_param_names(2),
        FunctionRepresentation::new_global_function("and_f"),
        FunctionInterface::new_operator(2, &bool_type, &bool_type),
    );
    add_function(&and_op, primitives::Type::Bool, PrimitiveOperation::And, module, runtime);

    let or__op = FunctionHead::new_static(
        FunctionHead::dummy_param_names(2),
        FunctionRepresentation::new_global_function("or_f"),
        FunctionInterface::new_operator(2, &bool_type, &bool_type),
    );
    add_function(&or__op, primitives::Type::Bool, PrimitiveOperation::Or, module, runtime);

    let not_op = FunctionHead::new_static(
        FunctionHead::dummy_param_names(1),
        FunctionRepresentation::new_global_function("not_f"),
        FunctionInterface::new_operator(1, &bool_type, &bool_type),
    );
    add_function(&not_op, primitives::Type::Bool, PrimitiveOperation::Not, module, runtime);
}
