//! A crate containing procedural macros for Luminary. These are used to reduce boilerplate and improve ergonomics within its codebase.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, Token, Type,
    parse::{Parse, ParseStream},
};
use syn::{ItemFn, parse_macro_input};

/// A macro to wrap the body of a function in `wrap_err`. This is useful for reducing boilerplate when using `eyre`.
#[proc_macro_attribute]
pub fn wrap_err(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let attr: TokenStream2 = attr.into();
    let visibility = &func.vis;
    let attrs = &func.attrs;

    // Create variables for quoting
    let asyncness = &func.sig.asyncness;
    let output = &func.sig.output;
    let signature = &func.sig;
    let block = &func.block;

    let wait = match asyncness.is_some() {
        true => Some(quote! { .await }),
        false => None,
    };

    return quote! {
        #(#attrs)*
        #visibility #signature {
            eyre::WrapErr::wrap_err((#asyncness move || #output #block)()#wait, #attr)
        }
    }
    .into();
}

/// A simple macro to obtain a type from the depot. Causing a panic if the type is not present.
#[proc_macro]
pub fn obtain(input: TokenStream) -> TokenStream {
    let ObtainArgs { ident, ty } = parse_macro_input!(input as ObtainArgs);
    let ty_str = quote! { #ty }.to_string();

    quote! {
        #ident.obtain::<#ty>()
            .expect(concat!(concat!("Tried to obtain an instance of ", #ty_str), " which the depot didn't have"))
    }
    .into()
}

struct ObtainArgs {
    ident: Expr,
    ty: Type,
}

impl Parse for ObtainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty: Type = input.parse()?;
        Ok(Self { ident, ty })
    }
}
