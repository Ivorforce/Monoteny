extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn bin_op(_item: TokenStream) -> TokenStream {
    let mut args_str = _item.to_string();  // TODO Stringifying back is stupid; is there a way to force the lexer to split just on spaces?
    let mut args = args_str.split(" ");

    let type_ = args.next().unwrap();
    let op = args.next().unwrap();
    let result_type = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            let layout = Layout::new::<{result_type}>();
            Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    // Avoid borrowing interpreter.
                    let args = interpreter.implementation.expression_tree.children[&expression_id].clone();
                    let l = interpreter.evaluate(args[0]).unwrap();
                    let r = interpreter.evaluate(args[1]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {result_type})) = *(l.data as *const {type_}) {op} *(r.data as *const {type_});
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}
    ", type_=type_, result_type=result_type, op=op).parse().unwrap()
}

#[proc_macro]
pub fn un_op(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let type_ = args.next().unwrap();
    let op = args.next().unwrap();

    format!("{{
            let layout = Layout::new::<{type_}>();
            Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_tree.children[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {type_})) = {op} *(arg.data as *const {type_});
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}", type_=type_, op=op).parse().unwrap()
}

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
            Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_tree.children[&expression_id].clone();
                    let l = interpreter.evaluate(args[0]).unwrap();
                    let r = interpreter.evaluate(args[1]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut {result_type})) = {result_type}::{op}(*(l.data as *const {type_}), *(r.data as *const {type_}));
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}
    ", type_=type_, result_type=result_type, op=op).parse().unwrap()
}

#[proc_macro]
pub fn parse_op(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let type_ = args.next().unwrap();

    format!("{{
            let layout = Layout::new::<{type_}>();
            Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_tree.children[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();
                    let data = alloc(layout);
                    *(data as *mut {type_}) = {type_}::from_str((*(arg.data as *const String)).as_str()).unwrap();
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}", type_=type_).parse().unwrap()
}

#[proc_macro]
pub fn to_string_op(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let type_ = args.next().unwrap();

    format!("{{
            let layout = Layout::new::<String>();
            Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_tree.children[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();

                    let data = alloc(layout);
                    (*(data as *mut String)) = (*(arg.data as *const {type_})).to_string();
                    return Some(Value {{ data, layout }});
                }}
            }})
        }}", type_=type_).parse().unwrap()
}

#[proc_macro]
pub fn load_constant(_item: TokenStream) -> TokenStream {
    let mut args = _item.into_iter();
    let type_ = args.next().unwrap();
    let value = args.next().unwrap();

    format!("{{
        let layout = Layout::new::<{type_}>();
        Rc::new(move |interpreter, expression_id, binding| {{
            unsafe {{
                let ptr = alloc(layout);
                *(ptr as *mut {type_}) = {value};
                return Some(Value {{ data: ptr, layout }})
            }}
        }})
    }}", type_=type_, value=value).parse().unwrap()
}
