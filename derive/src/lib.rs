#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Error};

mod subsume;

#[proc_macro_derive(Widen, attributes(subsume))]
/// Derive enum conversions. See the [widen API documentation](https://docs.rs/widen/latest/widen/derive.Widen.html).
pub fn derive_widen(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "Widen can only be derived for enums",
        ));
    };

    subsume::validate_attributes(&input.attrs)?;
    for variant in &data.variants {
        for field in &variant.fields {
            subsume::validate_attributes(&field.attrs)?;
        }
    }
    let from_impls = subsume::expand(&input, data)?;
    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let mut fields = variant.fields.iter();
            let (Some(field), None) = (fields.next(), fields.next()) else {
                return Err(Error::new_spanned(
                    variant,
                    "Widen requires each variant to contain exactly one field",
                ));
            };
            Ok((variant, field))
        })
        .collect::<syn::Result<Vec<_>>>();
    let variants = match variants {
        Ok(variants) => variants,
        Err(_) if !from_impls.is_empty() => return Ok(from_impls),
        Err(error) => return Err(error),
    };

    let path = match crate_name("widen").map_err(|error| Error::new(Span::call_site(), error))? {
        FoundCrate::Itself => quote!(::widen),
        FoundCrate::Name(name) => {
            let name = format_ident!("{name}");
            quote!(::#name)
        }
    };

    let mut target = format_ident!("__WidenTarget");
    while input
        .generics
        .type_params()
        .any(|param| param.ident == target)
        || input
            .generics
            .const_params()
            .any(|param| param.ident == target)
    {
        target = format_ident!("_{target}");
    }

    let mut generics = input.generics.clone();
    generics.params.push(parse_quote!(#target));
    let payload = format_ident!("__widen_payload", span = Span::mixed_site());
    let mut arms = Vec::new();
    for (variant, field) in variants {
        let ty = &field.ty;
        generics.make_where_clause().predicates.push(parse_quote!(
            #target: ::core::convert::From<#ty>
        ));

        let name = &variant.ident;
        let pattern = match &field.ident {
            Some(field) => quote!(Self::#name { #field: #payload }),
            None => quote!(Self::#name(#payload)),
        };
        arms.push(quote_spanned! {field.span()=>
            #pattern => <#target as ::core::convert::From<#ty>>::from(#payload)
        });
    }

    let name = &input.ident;
    let (_, type_generics, _) = input.generics.split_for_impl();
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    Ok(quote! {
        #from_impls

        impl #impl_generics #path::Widen<#target> for #name #type_generics #where_clause {
            fn widen(self) -> #target {
                match self { #(#arms,)* }
            }
        }
    })
}
