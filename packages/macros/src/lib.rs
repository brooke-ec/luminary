use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, FnArg, Ident, ItemFn,
    Pat, Token,
};

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);
    let visibility = &func.vis;

    // Extract locals to be deserialzed
    let mut locals = Punctuated::<FnArg, Token![,]>::new();
    for _ in 1..func.sig.inputs.len() {
        locals.push(func.sig.inputs.pop().unwrap().into_value());
    }

    // Push serde_json::Value argument to deserialize from
    func.sig.inputs.push(parse_quote!(value: serde_json::Value));

    // Generate the struct name for deserialization
    let struct_name = func.sig.ident.to_string().to_case(Case::Pascal) + "Command";
    let struct_ident = Ident::new(&struct_name, func.span());

    // Extract the identifiers of each local
    let locals_idents = locals.iter().map(|arg| match arg {
        FnArg::Typed(patype) => match patype.pat.as_ref() {
            Pat::Ident(ident) => &ident.ident,
            _ => panic!("Receiver arguments are not supported in commands"),
        },
        _ => panic!("Receiver arguments are not supported in commands"),
    });

    // Create variables for quoting
    let struct_properties = locals.iter();
    let signature = &func.sig;
    let block = &func.block;

    return quote! {
        #visibility #signature {
            let #struct_ident {#(#locals_idents),*} = serde_json::from_value::<#struct_ident>(value)?;

            #block
        }

        #[derive(serde::Deserialize, serde::Serialize)]
        #visibility struct #struct_ident {
            #(#struct_properties),*
        }
    }
    .into();
}

#[proc_macro]
pub fn route(item: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(item with Punctuated<Ident, Token![,]>::parse_terminated);

    return TokenStream::new();
}
