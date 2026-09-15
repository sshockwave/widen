#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Error, Fields};

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
    if !from_impls.is_empty()
        && data
            .variants
            .iter()
            .any(|variant| variant.fields.len() != 1)
    {
        return Ok(from_impls);
    }

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
    for variant in &data.variants {
        if variant.fields.len() != 1 {
            return Err(Error::new_spanned(
                variant,
                "Widen requires each variant to contain exactly one field",
            ));
        }
        let field = variant.fields.iter().next().expect("validated field count");
        let ty = &field.ty;
        generics.make_where_clause().predicates.push(parse_quote!(
            #target: ::core::convert::From<#ty>
        ));

        let name = &variant.ident;
        let pattern = match &variant.fields {
            Fields::Unnamed(_) => quote!(Self::#name(#payload)),
            Fields::Named(_) => {
                let field = &field.ident;
                quote!(Self::#name { #field: #payload })
            }
            Fields::Unit => unreachable!("validated field count"),
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
