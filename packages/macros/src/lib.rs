//! A crate containing procedural macros for Luminary. These are used to reduce boilerplate and improve ergonomics within its codebase.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
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
