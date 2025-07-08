extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn pop_ip(_item: TokenStream) -> TokenStream {
    let args_str = _item.to_string();
    let mut args = args_str.split(" ");

    let type_ = args.next().unwrap();
    assert!(args.next().is_none());

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            let val = read_unaligned(ip as *mut {type_});
            ip = transmute((ip as *mut {type_}).add(1));
            val
}}
    ", type_=type_).parse().unwrap()
}

#[proc_macro]
pub fn un_expr(_item: TokenStream) -> TokenStream {
    let args_str = _item.to_string();
    let mut args = args_str.split(",");

    let type_ = args.next().unwrap();
    let result_type = args.next().unwrap();
    let fun = args.collect::<Vec<_>>().join(",");

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            let sp_last = sp.offset(-8);
            let val = (*sp_last).{type_};
            (*sp_last).{result_type} = {fun};
}}
    ", type_=type_, result_type=result_type, fun=fun).parse().unwrap()
}

#[proc_macro]
pub fn un_expr_try(_item: TokenStream) -> TokenStream {
    let args_str = _item.to_string();
    let mut args = args_str.split(",");

    let type_ = args.next().unwrap();
    let result_type = args.next().unwrap();
    let fun = args.collect::<Vec<_>>().join(",");

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            let sp_last = sp.offset(-8);
            let val = (*sp_last).{type_};
            (*sp_last).{result_type} = {fun}?;
}}
    ", type_=type_, result_type=result_type, fun=fun).parse().unwrap()
}

#[proc_macro]
pub fn bin_expr(_item: TokenStream) -> TokenStream {
    let args_str = _item.to_string();
    let mut args = args_str.split(",");

    let type_ = args.next().unwrap();
    let result_type = args.next().unwrap();
    let fun = args.collect::<Vec<_>>().join(",");

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            sp = sp.offset(-8);
            let rhs = (*sp).{type_};

            let sp_last = sp.offset(-8);
            let lhs = (*sp_last).{type_};
            (*sp_last).{result_type} = {fun};
}}
    ", type_=type_, fun=fun, result_type=result_type).parse().unwrap()
}
