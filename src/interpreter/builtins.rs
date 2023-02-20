use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use monoteny_macro::{bin_op, parse_op, un_op, fun_op, load_constant, load_float_constant};
use std::str::FromStr;
use uuid::Uuid;
use crate::interpreter::{FunctionInterpreter, FunctionInterpreterImpl, Value};
use crate::program::builtins::Builtins;
use crate::program::functions::FunctionPointer;
use crate::program::primitives;
use crate::program::types::TypeUnit;

pub fn make_evaluators(builtins: &Builtins) -> HashMap<Uuid, FunctionInterpreterImpl> {
    let mut map: HashMap<Uuid, FunctionInterpreterImpl> = HashMap::new();

    let f32_type = TypeUnit::Struct(Rc::clone(&builtins.core.primitives[&primitives::Type::Float32]));
    let f64_type = TypeUnit::Struct(Rc::clone(&builtins.core.primitives[&primitives::Type::Float64]));

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Math --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Add --------------------------------------
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 + u8));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 + u16));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 + u32));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 + u64));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 + u128));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 + i8));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 + i16));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 + i32));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 + i64));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 + i128));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 + f32));
    map.insert(builtins.core.primitive_fns.add[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 + f64));

    // -------------------------------------- Subtract --------------------------------------
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 - u8));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 - u16));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 - u32));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 - u64));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 - u128));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 - i8));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 - i16));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 - i32));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 - i64));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 - i128));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 - f32));
    map.insert(builtins.core.primitive_fns.subtract[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 - f64));

    // -------------------------------------- Divide --------------------------------------
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 / u8));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 / u16));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 / u32));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 / u64));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 / u128));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 / i8));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 / i16));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 / i32));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 / i64));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 / i128));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 / f32));
    map.insert(builtins.core.primitive_fns.divide[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 / f64));

    // -------------------------------------- Multiply --------------------------------------
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 * u8));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 * u16));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 * u32));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 * u64));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 * u128));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 * i8));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 * i16));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 * i32));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 * i64));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 * i128));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 * f32));
    map.insert(builtins.core.primitive_fns.multiply[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 * f64));

    // -------------------------------------- Multiply --------------------------------------
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 % u8));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 % u16));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 % u32));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 % u64));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 % u128));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 % i8));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 % i16));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 % i32));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 % i64));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 % i128));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 % f32));
    map.insert(builtins.core.primitive_fns.modulo[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 % f64));

    // -------------------------------------- Floats --------------------------------------
    map.insert(builtins.core.primitive_fns.exponent[&primitives::Type::Float32].unwrap_id(), fun_op!(f32 powf f32));
    map.insert(builtins.core.primitive_fns.exponent[&primitives::Type::Float64].unwrap_id(), fun_op!(f64 powf f64));
    map.insert(builtins.core.primitive_fns.logarithm[&primitives::Type::Float32].unwrap_id(), fun_op!(f32 log f32));
    map.insert(builtins.core.primitive_fns.logarithm[&primitives::Type::Float64].unwrap_id(), fun_op!(f64 log f64));

    // -------------------------------------- Positive --------------------------------------
    // TODO Either remove positive op, or solve it as no-op. Either way, rust has no 'positive' op.

    // -------------------------------------- Negative --------------------------------------
    map.insert(builtins.core.primitive_fns.negative[&primitives::Type::Int8].unwrap_id(), un_op!(i8 -));
    map.insert(builtins.core.primitive_fns.negative[&primitives::Type::Int16].unwrap_id(), un_op!(i16 -));
    map.insert(builtins.core.primitive_fns.negative[&primitives::Type::Int32].unwrap_id(), un_op!(i32 -));
    map.insert(builtins.core.primitive_fns.negative[&primitives::Type::Int64].unwrap_id(), un_op!(i64 -));
    map.insert(builtins.core.primitive_fns.negative[&primitives::Type::Int128].unwrap_id(), un_op!(i128 -));
    map.insert(builtins.core.primitive_fns.negative[&primitives::Type::Float32].unwrap_id(), un_op!(f32 -));
    map.insert(builtins.core.primitive_fns.negative[&primitives::Type::Float64].unwrap_id(), un_op!(f64 -));

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Parsing --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Parse Int Literal --------------------------------------
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::UInt8].unwrap_id(), parse_op!(u8));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::UInt16].unwrap_id(), parse_op!(u16));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::UInt32].unwrap_id(), parse_op!(u32));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::UInt64].unwrap_id(), parse_op!(u64));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::UInt128].unwrap_id(), parse_op!(u128));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::Int8].unwrap_id(), parse_op!(i8));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::Int16].unwrap_id(), parse_op!(i16));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::Int32].unwrap_id(), parse_op!(i32));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::Int64].unwrap_id(), parse_op!(i64));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::Int128].unwrap_id(), parse_op!(i128));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::Float32].unwrap_id(), parse_op!(f32));
    map.insert(builtins.core.primitive_fns.parse_int_literal[&primitives::Type::Float64].unwrap_id(), parse_op!(f64));

    // -------------------------------------- Parse Float Literal --------------------------------------
    map.insert(builtins.core.primitive_fns.parse_float_literal[&primitives::Type::Float32].unwrap_id(), parse_op!(f32));
    map.insert(builtins.core.primitive_fns.parse_float_literal[&primitives::Type::Float64].unwrap_id(), parse_op!(f64));


    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Boolean --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Bool -> Bool --------------------------------------
    map.insert(builtins.core.primitive_fns.and.unwrap_id(), bin_op!(bool && bool));
    map.insert(builtins.core.primitive_fns.or.unwrap_id(), bin_op!(bool || bool));
    map.insert(builtins.core.primitive_fns.not.unwrap_id(), un_op!(bool !));

    // -------------------------------------- Equals --------------------------------------
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Bool].unwrap_id(), bin_op!(bool == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 == bool));
    map.insert(builtins.core.primitive_fns.equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 == bool));

    // -------------------------------------- Not Equals --------------------------------------
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Bool].unwrap_id(), bin_op!(bool != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 != bool));
    map.insert(builtins.core.primitive_fns.not_equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 != bool));

    // -------------------------------------- Greater Than --------------------------------------
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 > bool));
    map.insert(builtins.core.primitive_fns.greater_than[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 > bool));

    // -------------------------------------- Greater Than Or Equal To --------------------------------------
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 >= bool));
    map.insert(builtins.core.primitive_fns.greater_than_or_equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 >= bool));

    // -------------------------------------- Lesser Than --------------------------------------
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 < bool));
    map.insert(builtins.core.primitive_fns.lesser_than[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 < bool));

    // -------------------------------------- Lesser Than --------------------------------------
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 <= bool));
    map.insert(builtins.core.primitive_fns.lesser_than_or_equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 <= bool));

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Common --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    map.insert(builtins.debug.print.unwrap_id(), Box::new(|interpreter, expression_id, binding| {
        unsafe {
            let arg_id = &interpreter.function.expression_forest.arguments[expression_id][0];
            let arg = interpreter.evaluate(arg_id).unwrap();
            let arg_type = interpreter.function.type_forest.get_unit(arg_id).unwrap();

            // TODO Instead, introduce a ToString trait that can be called, with each getting their own function to fit it.
            //  If not implemented, dump the type instead.
            println!("{}", match arg_type {
                TypeUnit::Struct(s) => {
                    if s == &interpreter.builtins.core.traits.String {
                        (*(arg.data as *const String)).clone()
                    }
                    else if s == &interpreter.builtins.core.primitives[&primitives::Type::Bool] {
                        (*(arg.data as *const bool)).to_string()
                    }
                    else if s == &interpreter.builtins.core.primitives[&primitives::Type::Float32] {
                        (*(arg.data as *const f32)).to_string()
                    }
                    // TODO Transpile these too. Should probably make a lookup table somewhere?
                    // TypeUnit::Primitive(primitives::Type::Int8) => (*(arg.data as *const i8)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::Int16) => (*(arg.data as *const i16)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::Int32) => (*(arg.data as *const i32)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::Int64) => (*(arg.data as *const i64)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::Int128) => (*(arg.data as *const i128)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::UInt8) => (*(arg.data as *const u8)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::UInt16) => (*(arg.data as *const u16)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::UInt32) => (*(arg.data as *const u32)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::UInt64) => (*(arg.data as *const u64)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::UInt128) => (*(arg.data as *const u128)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::Float32) => (*(arg.data as *const f32)).to_string(),
                    // TypeUnit::Primitive(primitives::Type::Float64) => (*(arg.data as *const f64)).to_string(),
                    else {
                        panic!()
                    }
                },
                _ => panic!(),
            });

            return None;
        }
    }));

    map.insert(builtins.common.true_.unwrap_id(), load_constant!(bool true));
    map.insert(builtins.common.false_.unwrap_id(), load_constant!(bool false));

    map.insert(builtins.math.e.unwrap_id(), load_float_constant!(2.71828 f32_type f64_type));
    map.insert(builtins.math.pi.unwrap_id(), load_float_constant!(3.14159265 f32_type f64_type));
    map.insert(builtins.math.tau.unwrap_id(), load_float_constant!(1.57079633 f32_type f64_type));

    map
}
