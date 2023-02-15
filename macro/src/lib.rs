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
            Box::new(move |interpreter, expression_id| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let l = interpreter.evaluate(&args[0]).unwrap();
                    let r = interpreter.evaluate(&args[1]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {result_type})) = *(l.data as *mut {type_}) {op} *(r.data as *mut {type_});
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
            Box::new(move |interpreter, expression_id| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let arg = interpreter.evaluate(&args[0]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {type_})) = {op} *(arg.data as *mut {type_});
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
            Box::new(move |interpreter, expression_id| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let l = interpreter.evaluate(&args[0]).unwrap();
                    let r = interpreter.evaluate(&args[1]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {result_type})) = {result_type}::{op}(*(l.data as *mut {type_}), *(r.data as *mut {type_}));
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
            Box::new(move |interpreter, expression_id| {{
                unsafe {{
                    let args = &interpreter.function.expression_forest.arguments[expression_id];
                    let arg = interpreter.evaluate(&args[0]).unwrap();
                    let data = alloc(layout);
                    *(data as *mut {type_}) = {type_}::from_str((*(arg.data as *mut String)).as_str()).unwrap();
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}", type_=type_).parse().unwrap()
}
