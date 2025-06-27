use heck::ToSnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse2, Attribute, Data, DeriveInput, Error, Expr, Fields, FieldsNamed, LitInt, Result};

use crate::{add_trait_bounds, pair_variants_with_discriminants};

pub(super) fn derive_hash_ops(item: TokenStream) -> Result<TokenStream> {
    let mut input = parse2::<DeriveInput>(item)?;

    let input_name = input.ident;

    add_trait_bounds(
        &mut input.generics,
        quote!(::valence_protocol::hash_utils::HashOpsHashable),
    );

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match input.data {
        Data::Struct(struct_) => {
            let encode_fields = match &struct_.fields {
                Fields::Named(fields) => process_named_fields(fields, true).1,
                Fields::Unnamed(fields) => (0..fields.unnamed.len())
                    .map(|i| {
                        let lit = LitInt::new(&i.to_string(), Span::call_site());
                        quote! {
                            HashOpsHashable::hash(self.#lit, hasher);
                        }
                    })
                    .collect(),
                Fields::Unit => TokenStream::new(),
            };

            Ok(quote! {
                #[allow(unused_imports)]
                impl #impl_generics ::valence_protocol::hash_utils::HashOpsHashable for #input_name #ty_generics
                #where_clause
                {
                    fn hash<T>(&self, hasher: &mut T)
                     where
                         T: ::std::hash::Hasher + Sized,
                     {
                        use ::valence_protocol::{hash_utils::{HashOpsHashable, HashOps, HashCode}, Context};

                        #encode_fields
                    }
                }
            })
        }
        Data::Enum(enum_) => {
            let variants = pair_variants_with_discriminants(enum_.variants)?;

            let is_variant = variants
                .iter()
                .all(|(_, v)| matches!(v.fields, Fields::Unit));

            let encode_arms = variants
                .iter()
                .map(|(disc, variant)| {

                    let variant_name = &variant.ident;

                    match &variant.fields {
                        Fields::Named(fields) => {
                            let (names, hash_code) = process_named_fields(fields, false);

                            quote! {
                                Self::#variant_name { #(#names,)* } => {
                                    #hash_code
                                }
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let field_names = (0..fields.unnamed.len())
                                .map(|i| Ident::new(&format!("_{i}"), Span::call_site()))
                                .collect::<Vec<_>>();

                            let encode_fields = field_names
                                .iter()
                                .map(|name| {
                                    quote! {
                                        HashOpsHashable::hash(#name, hasher);
                                    }
                                })
                                .collect::<TokenStream>();

                            quote! {
                                Self::#variant_name(#(#field_names,)*) => {
                                    #encode_fields
                                }
                            }
                        }
                        Fields::Unit => if is_variant {
                            quote! {
                               Self::#variant_name =>  HashOpsHashable::hash(&#disc, hasher),
                            }
                        } else {
                            quote! {
                                // Hash as empty map
                                Self::#variant_name => HashOpsHashable::hash(&Vec::<(i8, i8)>::with_capacity(0), hasher),
                            }
                        },
                    }
                })
                .collect::<TokenStream>();

            Ok(quote! {
                #[allow(unused_imports, unreachable_code)]
                impl #impl_generics ::valence_protocol::hash_utils::HashOpsHashable for #input_name #ty_generics
                #where_clause
                {
                     fn hash<T>(&self, hasher: &mut T)
                     where
                         T: ::std::hash::Hasher + Sized,
                     {
                        use ::valence_protocol::{hash_utils::{HashOpsHashable, HashOps, HashCode}, VarInt, Context};

                        match self {
                            #encode_arms
                            _ => unreachable!(),
                        }
                    }
                }
            })
        }
        Data::Union(u) => Err(Error::new(
            u.union_token.span(),
            "cannot derive `HashOps` on unions",
        )),
    }
}

fn process_named_fields(fields: &FieldsNamed, is_self: bool) -> (Vec<&Ident>, TokenStream) {
    let names = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect::<Vec<_>>();
    let fields: Vec<TokenStream> = fields.named
        .iter()
        .map(| field| {
            let attr = parse_attribute(&field.attrs).unwrap();
            let name = &field.ident.as_ref().unwrap();
            let raw_name = name.to_string().to_snake_case();
            let name = match is_self {
                true => quote!(&self.#name),
                false => quote!(#name),
            };
            let hashed_field = quote! {
                __data.push((
                    HashCode::new(HashOps::hash(&#raw_name)),
                    HashCode::new(HashOps::hash(#name))
                ));
            };
            match attr {
                Some(attr) if attr.option.is_some()  => {
                    let expr = attr.option.as_ref().unwrap();
                    quote! {
                        if #name != &#expr {
                            #hashed_field
                        }
                    }
                },
                _ => hashed_field,
            }
        })
        .collect();
    let len = names.len();
    let tokens = quote! {
        let mut __data: Vec<(HashCode, HashCode)> = Vec::with_capacity(#len);
        #(#fields)*
        __data.sort_by(HashCode::sort);
        HashOpsHashable::hash(&__data, hasher);
    };
    (names, tokens)
}

struct HashOpsAttribute {
    option: Option<Expr>,
}
fn parse_attribute(attrs: &[Attribute]) -> Result<Option<HashOpsAttribute>> {
    for attr in attrs {
        if !attr.path().is_ident("hash_ops") {
            continue
        }

        let mut res = HashOpsAttribute { option: None };

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("option") {
                res.option = Some(meta.value()?.parse::<Expr>()?);
                Ok(())
            } else {
                Err(meta.error("unrecognized hash_ops argument"))
            }
        })?;

        return Ok(Some(res))

    }
    Ok(None)
}
