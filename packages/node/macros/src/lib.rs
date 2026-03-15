//! A crate containing procedural macros for Luminary. These are used to reduce boilerplate and improve ergonomics within its codebase.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemFn, ItemStruct, parse_macro_input};

/// A macro to wrap the body of a function in `wrap_err`. This is useful for reducing boilerplate when using `eyre`.
#[proc_macro_attribute]
pub fn wrap_err(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let attr: TokenStream2 = attr.into();

    // Create variables for quoting
    let visibility = &func.vis;
    let attrs = &func.attrs;
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

/// Creates a transparent wrapper
///
/// # Examples
/// ```
/// #[hashmap_schema]
/// pub struct StructName<String, MyType>;
/// ```
#[proc_macro_attribute]
pub fn hashmap_schema(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    if item.fields.len() != 0 {
        return syn::Error::new_spanned(item.fields, "expected a struct with no fields")
            .to_compile_error()
            .into();
    }

    let mut generics: Vec<_> = item
        .generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(ty) => {
                let ident = &ty.ident;
                quote! { #ident }
            }
            generic => {
                return syn::Error::new_spanned(generic, "expected only type generics")
                    .to_compile_error()
                    .into();
            }
        })
        .collect();

    if generics.len() != 2 {
        return syn::Error::new_spanned(item.generics, "expected exactly two type generics")
            .to_compile_error()
            .into();
    }

    // Create variables for quoting
    let value = generics.pop().unwrap();
    let key = generics.pop().unwrap();
    let visibility = &item.vis;
    let attrs = &item.attrs;
    let ident = &item.ident;

    quote! {
        #(#attrs)*
        #[serde(transparent)]
        #visibility struct #ident(pub std::collections::HashMap<#key, #value>);

        impl #ident {
            pub fn new() -> Self {
                Self(std::collections::HashMap::new())
            }
        }

        impl std::ops::Deref for #ident {
            type Target = std::collections::HashMap<#key, #value>;

            fn deref<'a>(&'a self) -> &'a Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for #ident {
            fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target {
                &mut self.0
            }
        }

        impl std::convert::From<std::collections::HashMap<#key, #value>> for #ident {
            fn from(value: std::collections::HashMap<#key, #value>) -> Self {
                Self(value)
            }
        }

        impl std::convert::From<#ident> for std::collections::HashMap<#key, #value> {
            fn from(value: #ident) -> Self {
                value.0
            }
        }

        impl std::iter::IntoIterator for #ident {
            type Item = (#key, #value);
            type IntoIter = std::collections::hash_map::IntoIter<#key, #value>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a> std::iter::IntoIterator for &'a #ident {
            type Item = (&'a #key, &'a #value);
            type IntoIter = std::collections::hash_map::Iter<'a, #key, #value>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl<'a> std::iter::IntoIterator for &'a mut #ident {
            type Item = (&'a #key, &'a mut #value);
            type IntoIter = std::collections::hash_map::IterMut<'a, #key, #value>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter_mut()
            }
        }

        impl salvo::oapi::ToSchema for #ident {
            fn to_schema(
                components: &mut salvo::oapi::Components,
            ) -> salvo::oapi::RefOr<::salvo::oapi::schema::Schema> {
                let name = salvo::oapi::naming::assign_name::<Self>(::std::default::Default::default());
                let ref_or = salvo::oapi::RefOr::Ref(::salvo::oapi::Ref::new(format!("#/components/schemas/{}", name)));

                if !components.schemas.contains_key(&name) {
                    components.schemas.insert(name.clone(), ref_or.clone());
                    let schema = salvo::oapi::Schema::Object(
                        salvo::oapi::Object::new()
                            .additional_properties(<#value as salvo::oapi::ToSchema>::to_schema(components))
                            .into(),
                    );
                    components.schemas.insert(name, schema);
                }

                ref_or
            }
        }
    }
    .into()
}
