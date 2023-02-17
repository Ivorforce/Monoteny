extern crate proc_macro;
use proc_macro::TokenStream;

#[macro_export]
#[proc_macro]
pub fn bin_op(_item: TokenStream) -> TokenStream {
    let args_str = _item.to_string(); // TODO Stringifying back is stupid; is there a way to force the lexer to split just on spaces?
    let mut args = args_str.split(" ");
    let type_ = args.next().unwrap();
    let op = args.next().unwrap();
    let result_type = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            let layout = Layout::new::<{result_type}>();
            Box::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let l = interpreter.evaluate(&args[0]).unwrap();
                    let r = interpreter.evaluate(&args[1]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {result_type})) = *(l.data as *const {type_}) {op} *(r.data as *const {type_});
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}
    ", type_=type_, result_type=result_type, op=op).parse().unwrap()
}

#[macro_export]
#[proc_macro]
pub fn un_op(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let type_ = args.next().unwrap();
    let op = args.next().unwrap();

    format!("{{
            let layout = Layout::new::<{type_}>();
            Box::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let arg = interpreter.evaluate(&args[0]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {type_})) = {op} *(arg.data as *const {type_});
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}", type_=type_, op=op).parse().unwrap()
}

#[macro_export]
#[proc_macro]
pub fn fun_op(_item: TokenStream) -> TokenStream {
    let args_str = _item.to_string(); // TODO Stringifying back is stupid; is there a way to force the lexer to split just on spaces?
    let mut args = args_str.split(" ");
    let type_ = args.next().unwrap();
    let op = args.next().unwrap();
    let result_type = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            let layout = Layout::new::<{result_type}>();
            Box::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let l = interpreter.evaluate(&args[0]).unwrap();
                    let r = interpreter.evaluate(&args[1]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {result_type})) = {result_type}::{op}(*(l.data as *const {type_}), *(r.data as *const {type_}));
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}
    ", type_=type_, result_type=result_type, op=op).parse().unwrap()
}

#[macro_export]
#[proc_macro]
pub fn parse_op(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let type_ = args.next().unwrap();

    format!("{{
            let layout = Layout::new::<{type_}>();
            Box::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let arg = interpreter.evaluate(&args[0]).unwrap();
                    let data = alloc(layout);
                    *(data as *mut {type_}) = {type_}::from_str((*(arg.data as *const String)).as_str()).unwrap();
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}", type_=type_).parse().unwrap()
}

#[macro_export]
#[proc_macro]
pub fn load_constant(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let type_ = args.next().unwrap();
    let value = args.next().unwrap();

    format!("{{
        let layout = Layout::new::<{type_}>();
        Box::new(move |interpreter, expression_id, binding| {{
            unsafe {{
                let ptr = alloc(layout);
                *(ptr as *mut {type_}) = {value};
                return Some(Value {{ data: ptr, layout }})
            }}
        }})
    }}", type_=type_, value=value).parse().unwrap()
}

#[macro_export]
#[proc_macro]
pub fn load_float_constant(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let value = args.next().unwrap();

    format!("{{
        let f32_layout = Layout::new::<f32>();
        let f64_layout = Layout::new::<f64>();

        Box::new(move |interpreter, expression_id, binding| {{
            unsafe {{
                let return_type = interpreter.function.type_forest.get_unit(expression_id).unwrap();

                match return_type {{
                    TypeUnit::Primitive(primitives::Type::Float32) => {{
                        let ptr = alloc(f32_layout);
                        *(ptr as *mut f32) = {value};
                        return Some(Value {{ data: ptr, layout: f32_layout }})
                    }}
                    TypeUnit::Primitive(primitives::Type::Float64) => {{
                        let ptr = alloc(f64_layout);
                        *(ptr as *mut f64) = {value};
                        return Some(Value {{ data: ptr, layout: f64_layout }})
                    }}
                    _ => panic!(\"Non-float type\")
                }}
            }}
        }})
    }}", value=value).parse().unwrap()
}
