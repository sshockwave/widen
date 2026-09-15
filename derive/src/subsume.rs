use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, DataEnum, DeriveInput, Error, Fields, Ident, Path, PathArguments, Token,
};

struct Mapping {
    path: Path,
    convert: bool,
}

impl Parse for Mapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let convert = input.peek(Ident) && input.peek2(token::Paren);
        let path = if convert {
            let modifier: Ident = input.parse()?;
            if modifier != "from" {
                return Err(Error::new_spanned(
                    modifier,
                    "expected `from(Source::Variant)`",
                ));
            }
            let content;
            parenthesized!(content in input);
            let path = content.parse()?;
            if !content.is_empty() {
                return Err(content.error("expected a single source variant"));
            }
            path
        } else {
            input.parse()?
        };
        Ok(Self { path, convert })
    }
}

struct Source {
    ty: Path,
    variants: BTreeSet<String>,
    arms: Vec<TokenStream>,
}

pub(super) fn validate_attributes(attrs: &[Attribute]) -> syn::Result<()> {
    for attr in attrs {
        if attr.path().is_ident("subsume") {
            return Err(Error::new_spanned(
                attr,
                "`subsume` must be placed on an enum variant",
            ));
        }
    }
    Ok(())
}

pub(super) fn expand(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let mut sources = BTreeMap::<String, Source>::new();
    for variant in &data.variants {
        for attr in variant
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("subsume"))
        {
            let mappings =
                attr.parse_args_with(Punctuated::<Mapping, Token![,]>::parse_terminated)?;
            if mappings.is_empty() {
                return Err(Error::new_spanned(
                    attr,
                    "expected at least one source variant",
                ));
            }
            for mapping in mappings {
                let mut path = mapping.path;
                if path.segments.len() < 2 {
                    return Err(Error::new_spanned(path, "expected `Source::Variant`"));
                }
                let mut ty = path.clone();
                let source_variant = ty
                    .segments
                    .pop()
                    .expect("validated path length")
                    .into_value();
                ty.segments.pop_punct();
                if !matches!(source_variant.arguments, PathArguments::None) {
                    return Err(Error::new_spanned(
                        source_variant,
                        "put generic arguments on the source enum",
                    ));
                }
                // Keep generics on the source type; patterns infer them from the input.
                for segment in &mut ty.segments {
                    if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                        args.colon2_token = None;
                    }
                }
                for segment in &mut path.segments {
                    segment.arguments = PathArguments::None;
                }
                let key = quote!(#ty).to_string();
                let source = sources.entry(key).or_insert_with(|| Source {
                    ty,
                    variants: BTreeSet::new(),
                    arms: Vec::new(),
                });
                if !source.variants.insert(source_variant.ident.to_string()) {
                    return Err(Error::new_spanned(path, "source variant is already mapped"));
                }
                if mapping.convert && variant.fields.is_empty() {
                    return Err(Error::new_spanned(
                        path,
                        "`from(...)` requires a variant with fields",
                    ));
                }

                let bindings: Vec<_> = variant
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(index, _)| {
                        format_ident!("__widen_field_{index}", span = Span::mixed_site())
                    })
                    .collect();
                let values: Vec<_> = bindings
                    .iter()
                    .map(|binding| {
                        if mapping.convert {
                            quote_spanned!(path.span()=> ::core::convert::Into::into(#binding))
                        } else {
                            quote!(#binding)
                        }
                    })
                    .collect();
                let name = &variant.ident;
                let (pattern, value) = match &variant.fields {
                    Fields::Unit => (quote!(#path), quote!(Self::#name)),
                    Fields::Unnamed(_) => (
                        quote!(#path(#(#bindings),*)),
                        quote!(Self::#name(#(#values),*)),
                    ),
                    Fields::Named(fields) => {
                        let names: Vec<_> = fields.named.iter().map(|field| &field.ident).collect();
                        (
                            quote!(#path { #(#names: #bindings),* }),
                            quote!(Self::#name { #(#names: #values),* }),
                        )
                    }
                };
                source
                    .arms
                    .push(quote_spanned!(path.span()=> #pattern => #value));
            }
        }
    }

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let impls = sources.into_values().map(|Source { ty, arms, .. }| {
        quote! {
            impl #impl_generics ::core::convert::From<#ty> for #name #type_generics #where_clause {
                fn from(source: #ty) -> Self {
                    match source { #(#arms,)* }
                }
            }
        }
    });
    Ok(quote!(#(#impls)*))
}
