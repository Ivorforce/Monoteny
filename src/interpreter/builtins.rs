use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use monoteny_macro::{bin_op, parse_op, un_op, fun_op};
use std::str::FromStr;
use crate::interpreter::{FunctionInterpreter, FunctionInterpreterImpl, Value};
use crate::program::builtins::Builtins;
use crate::program::functions::FunctionPointer;
use crate::program::primitives;
use crate::program::types::TypeUnit;

pub fn make_evaluators(builtins: &Builtins) -> HashMap<Rc<FunctionPointer>, FunctionInterpreterImpl> {
    let mut map: HashMap<Rc<FunctionPointer>, FunctionInterpreterImpl> = HashMap::new();

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Math --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Add --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::UInt8]), bin_op!(u8 + u8));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::UInt16]), bin_op!(u16 + u16));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::UInt32]), bin_op!(u32 + u32));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::UInt64]), bin_op!(u64 + u64));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::UInt128]), bin_op!(u128 + u128));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::Int8]), bin_op!(i8 + i8));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::Int16]), bin_op!(i16 + i16));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::Int32]), bin_op!(i32 + i32));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::Int64]), bin_op!(i64 + i64));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::Int128]), bin_op!(i128 + i128));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::Float32]), bin_op!(f32 + f32));
    map.insert(Rc::clone(&builtins.primitives.add[&primitives::Type::Float64]), bin_op!(f64 + f64));

    // -------------------------------------- Subtract --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::UInt8]), bin_op!(u8 - u8));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::UInt16]), bin_op!(u16 - u16));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::UInt32]), bin_op!(u32 - u32));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::UInt64]), bin_op!(u64 - u64));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::UInt128]), bin_op!(u128 - u128));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::Int8]), bin_op!(i8 - i8));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::Int16]), bin_op!(i16 - i16));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::Int32]), bin_op!(i32 - i32));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::Int64]), bin_op!(i64 - i64));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::Int128]), bin_op!(i128 - i128));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::Float32]), bin_op!(f32 - f32));
    map.insert(Rc::clone(&builtins.primitives.subtract[&primitives::Type::Float64]), bin_op!(f64 - f64));

    // -------------------------------------- Divide --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::UInt8]), bin_op!(u8 / u8));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::UInt16]), bin_op!(u16 / u16));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::UInt32]), bin_op!(u32 / u32));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::UInt64]), bin_op!(u64 / u64));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::UInt128]), bin_op!(u128 / u128));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::Int8]), bin_op!(i8 / i8));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::Int16]), bin_op!(i16 / i16));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::Int32]), bin_op!(i32 / i32));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::Int64]), bin_op!(i64 / i64));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::Int128]), bin_op!(i128 / i128));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::Float32]), bin_op!(f32 / f32));
    map.insert(Rc::clone(&builtins.primitives.divide[&primitives::Type::Float64]), bin_op!(f64 / f64));

    // -------------------------------------- Multiply --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::UInt8]), bin_op!(u8 * u8));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::UInt16]), bin_op!(u16 * u16));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::UInt32]), bin_op!(u32 * u32));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::UInt64]), bin_op!(u64 * u64));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::UInt128]), bin_op!(u128 * u128));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::Int8]), bin_op!(i8 * i8));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::Int16]), bin_op!(i16 * i16));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::Int32]), bin_op!(i32 * i32));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::Int64]), bin_op!(i64 * i64));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::Int128]), bin_op!(i128 * i128));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::Float32]), bin_op!(f32 * f32));
    map.insert(Rc::clone(&builtins.primitives.multiply[&primitives::Type::Float64]), bin_op!(f64 * f64));

    // -------------------------------------- Multiply --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::UInt8]), bin_op!(u8 % u8));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::UInt16]), bin_op!(u16 % u16));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::UInt32]), bin_op!(u32 % u32));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::UInt64]), bin_op!(u64 % u64));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::UInt128]), bin_op!(u128 % u128));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::Int8]), bin_op!(i8 % i8));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::Int16]), bin_op!(i16 % i16));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::Int32]), bin_op!(i32 % i32));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::Int64]), bin_op!(i64 % i64));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::Int128]), bin_op!(i128 % i128));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::Float32]), bin_op!(f32 % f32));
    map.insert(Rc::clone(&builtins.primitives.modulo[&primitives::Type::Float64]), bin_op!(f64 % f64));

    // -------------------------------------- Floats --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.exponent[&primitives::Type::Float32]), fun_op!(f32 powf f32));
    map.insert(Rc::clone(&builtins.primitives.exponent[&primitives::Type::Float64]), fun_op!(f64 powf f64));
    map.insert(Rc::clone(&builtins.primitives.logarithm[&primitives::Type::Float32]), fun_op!(f32 log f32));
    map.insert(Rc::clone(&builtins.primitives.logarithm[&primitives::Type::Float64]), fun_op!(f64 log f64));

    // -------------------------------------- Positive --------------------------------------
    // TODO Either remove positive op, or solve it as no-op. Either way, rust has no 'positive' op.

    // -------------------------------------- Negative --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.negative[&primitives::Type::Int8]), un_op!(i8 -));
    map.insert(Rc::clone(&builtins.primitives.negative[&primitives::Type::Int16]), un_op!(i16 -));
    map.insert(Rc::clone(&builtins.primitives.negative[&primitives::Type::Int32]), un_op!(i32 -));
    map.insert(Rc::clone(&builtins.primitives.negative[&primitives::Type::Int64]), un_op!(i64 -));
    map.insert(Rc::clone(&builtins.primitives.negative[&primitives::Type::Int128]), un_op!(i128 -));
    map.insert(Rc::clone(&builtins.primitives.negative[&primitives::Type::Float32]), un_op!(f32 -));
    map.insert(Rc::clone(&builtins.primitives.negative[&primitives::Type::Float64]), un_op!(f64 -));

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Parsing --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Parse Int Literal --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::UInt8]), parse_op!(u8));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::UInt16]), parse_op!(u16));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::UInt32]), parse_op!(u32));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::UInt64]), parse_op!(u64));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::UInt128]), parse_op!(u128));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::Int8]), parse_op!(i8));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::Int16]), parse_op!(i16));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::Int32]), parse_op!(i32));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::Int64]), parse_op!(i64));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::Int128]), parse_op!(i128));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::Float32]), parse_op!(f32));
    map.insert(Rc::clone(&builtins.primitives.parse_int_literal[&primitives::Type::Float64]), parse_op!(f64));

    // -------------------------------------- Parse Float Literal --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.parse_float_literal[&primitives::Type::Float32]), parse_op!(f32));
    map.insert(Rc::clone(&builtins.primitives.parse_float_literal[&primitives::Type::Float64]), parse_op!(f64));


    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Boolean --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Bool -> Bool --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.and), bin_op!(bool && bool));
    map.insert(Rc::clone(&builtins.primitives.or), bin_op!(bool || bool));
    map.insert(Rc::clone(&builtins.primitives.not), un_op!(bool !));

    // -------------------------------------- Equals --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Bool]), bin_op!(bool == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::UInt8]), bin_op!(u8 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::UInt16]), bin_op!(u16 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::UInt32]), bin_op!(u32 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::UInt64]), bin_op!(u64 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::UInt128]), bin_op!(u128 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Int8]), bin_op!(i8 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Int16]), bin_op!(i16 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Int32]), bin_op!(i32 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Int64]), bin_op!(i64 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Int128]), bin_op!(i128 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Float32]), bin_op!(f32 == bool));
    map.insert(Rc::clone(&builtins.primitives.equal_to[&primitives::Type::Float64]), bin_op!(f64 == bool));

    // -------------------------------------- Not Equals --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Bool]), bin_op!(bool != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::UInt8]), bin_op!(u8 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::UInt16]), bin_op!(u16 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::UInt32]), bin_op!(u32 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::UInt64]), bin_op!(u64 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::UInt128]), bin_op!(u128 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Int8]), bin_op!(i8 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Int16]), bin_op!(i16 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Int32]), bin_op!(i32 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Int64]), bin_op!(i64 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Int128]), bin_op!(i128 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Float32]), bin_op!(f32 != bool));
    map.insert(Rc::clone(&builtins.primitives.not_equal_to[&primitives::Type::Float64]), bin_op!(f64 != bool));

    // -------------------------------------- Greater Than --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::UInt8]), bin_op!(u8 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::UInt16]), bin_op!(u16 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::UInt32]), bin_op!(u32 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::UInt64]), bin_op!(u64 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::UInt128]), bin_op!(u128 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::Int8]), bin_op!(i8 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::Int16]), bin_op!(i16 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::Int32]), bin_op!(i32 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::Int64]), bin_op!(i64 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::Int128]), bin_op!(i128 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::Float32]), bin_op!(f32 > bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than[&primitives::Type::Float64]), bin_op!(f64 > bool));

    // -------------------------------------- Greater Than Or Equal To --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt8]), bin_op!(u8 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt16]), bin_op!(u16 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt32]), bin_op!(u32 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt64]), bin_op!(u64 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt128]), bin_op!(u128 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int8]), bin_op!(i8 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int16]), bin_op!(i16 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int32]), bin_op!(i32 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int64]), bin_op!(i64 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int128]), bin_op!(i128 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::Float32]), bin_op!(f32 >= bool));
    map.insert(Rc::clone(&builtins.primitives.greater_than_or_equal_to[&primitives::Type::Float64]), bin_op!(f64 >= bool));

    // -------------------------------------- Lesser Than --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::UInt8]), bin_op!(u8 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::UInt16]), bin_op!(u16 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::UInt32]), bin_op!(u32 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::UInt64]), bin_op!(u64 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::UInt128]), bin_op!(u128 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::Int8]), bin_op!(i8 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::Int16]), bin_op!(i16 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::Int32]), bin_op!(i32 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::Int64]), bin_op!(i64 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::Int128]), bin_op!(i128 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::Float32]), bin_op!(f32 < bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than[&primitives::Type::Float64]), bin_op!(f64 < bool));

    // -------------------------------------- Lesser Than --------------------------------------
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt8]), bin_op!(u8 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt16]), bin_op!(u16 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt32]), bin_op!(u32 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt64]), bin_op!(u64 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt128]), bin_op!(u128 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int8]), bin_op!(i8 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int16]), bin_op!(i16 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int32]), bin_op!(i32 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int64]), bin_op!(i64 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int128]), bin_op!(i128 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Float32]), bin_op!(f32 <= bool));
    map.insert(Rc::clone(&builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Float64]), bin_op!(f64 <= bool));

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Common --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    map.insert(Rc::clone(&builtins.debug.print), Box::new(|interpreter, expression_id| {
        unsafe {
            let arg_id = &interpreter.function.expression_forest.arguments[expression_id][0];
            let arg = interpreter.evaluate(arg_id).unwrap();
            let arg_type = interpreter.function.type_forest.get_unit(arg_id).unwrap();

            // TODO Instead, introduce a ToString trait that can be called, with each getting their own function to fit it.
            //  If not implemented, dump the type instead.
            println!("{}", match arg_type {
                TypeUnit::Struct(s) => {
                    if s == &interpreter.builtins.traits.String {
                        (*(arg.data as *mut String)).clone()
                    }
                    else {
                        panic!()
                    }
                },
                TypeUnit::Primitive(primitives::Type::Bool) => (*(arg.data as *mut bool)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int8) => (*(arg.data as *mut i8)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int16) => (*(arg.data as *mut i16)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int32) => (*(arg.data as *mut i32)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int64) => (*(arg.data as *mut i64)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int128) => (*(arg.data as *mut i128)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt8) => (*(arg.data as *mut u8)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt16) => (*(arg.data as *mut u16)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt32) => (*(arg.data as *mut u32)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt64) => (*(arg.data as *mut u64)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt128) => (*(arg.data as *mut u128)).to_string(),
                TypeUnit::Primitive(primitives::Type::Float32) => (*(arg.data as *mut f32)).to_string(),
                TypeUnit::Primitive(primitives::Type::Float64) => (*(arg.data as *mut f64)).to_string(),
                _ => panic!(),
            });

            return None;
        }
    }));

    let bool_layout = Layout::new::<bool>();
    map.insert(Rc::clone(&builtins.common.true_), Box::new(move |interpreter, expression_id| {
        unsafe {
            let ptr = alloc(bool_layout);
            *(ptr as *mut bool) = true;
            return Some(Value { data: ptr, layout: bool_layout })
        }
    }));
    map.insert(Rc::clone(&builtins.common.false_), Box::new(move |interpreter, expression_id| {
        unsafe {
            let ptr = alloc(bool_layout);
            *(ptr as *mut bool) = false;
            return Some(Value { data: ptr, layout: bool_layout })
        }
    }));

    map
}
