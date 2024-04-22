extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn load_ip(_item: TokenStream) -> TokenStream {
    let mut args_str = _item.to_string();
    let mut args = args_str.split(" ");

    let ip = args.next().unwrap();
    let sp = args.next().unwrap();
    let type_ = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            {ip} = {ip}.add(1);
            write_unaligned({sp} as *mut {type_}, read_unaligned({ip} as *mut {type_}));
            {ip} = transmute(({ip} as *mut {type_}).add(1));
            {sp} = transmute(({sp} as *mut {type_}).add(1));
}}
    ", ip=ip, sp=sp, type_=type_).parse().unwrap()
}

#[proc_macro]
pub fn bin_op(_item: TokenStream) -> TokenStream {
    let mut args_str = _item.to_string();
    let mut args = args_str.split(" ");

    let sp = args.next().unwrap();
    let type_ = args.next().unwrap();
    let op = args.next().unwrap();

    // TODO When it's stable, these should be replaced by quote!()
    format!("
{{
            {sp} = transmute(({sp} as *mut {type_}).offset(-1));
            let rhs = *({sp} as *mut {type_});

            let sp_last = ({sp} as *mut {type_}).offset(-1);
            let lhs = *sp_last;
            *sp_last = lhs {op} rhs;
}}
    ", sp=sp, type_=type_, op=op).parse().unwrap()
}
