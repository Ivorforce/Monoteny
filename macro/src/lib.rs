extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn pop_ip(_item: TokenStream) -> TokenStream {
    let mut args_str = _item.to_string();
    let mut args = args_str.split(" ");

    let type_ = args.next().unwrap();

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
pub fn pop_sp(_item: TokenStream) -> TokenStream {
    assert!(_item.is_empty());

    format!("
{{
            sp = sp.offset(-8);
            *sp
}}
    ").parse().unwrap()
}

#[proc_macro]
pub fn bin_op(_item: TokenStream) -> TokenStream {
    let mut args_str = _item.to_string();
    let mut args = args_str.split(" ");

    let type_ = args.next().unwrap();
    let op = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            sp = sp.offset(-8);
            let rhs = (*sp).{type_};

            let sp_last = sp.offset(-8);
            let lhs = (*sp_last).{type_};
            (*sp_last).{type_} = lhs {op} rhs;
}}
    ", type_=type_, op=op).parse().unwrap()
}

#[proc_macro]
pub fn bool_bin_op(_item: TokenStream) -> TokenStream {
    let mut args_str = _item.to_string();
    let mut args = args_str.split(" ");

    let op = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            sp = sp.offset(-8);
            let rhs = (*sp).bool;

            let sp_last = sp.offset(-8);
            let lhs = (*sp_last).bool;
            (*sp_last).bool = lhs {op} rhs;
}}
    ", op=op).parse().unwrap()
}

#[proc_macro]
pub fn to_bool_bin_op(_item: TokenStream) -> TokenStream {
    let mut args_str = _item.to_string();
    let mut args = args_str.split(" ");

    let type_ = args.next().unwrap();
    let op = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            sp = sp.offset(-8);
            let rhs = (*sp).{type_};

            sp = sp.offset(-8);
            let lhs = (*sp).{type_};

            (*sp).bool = lhs {op} rhs;
            sp = sp.add(8);
}}
    ", type_=type_, op=op).parse().unwrap()
}
