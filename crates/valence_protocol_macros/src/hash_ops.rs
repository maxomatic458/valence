use heck::ToSnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse2, Data, DeriveInput, Error, Fields, LitInt, Result};

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
                Fields::Named(fields) => {
                    let fields: Vec<TokenStream> = fields
                        .named
                        .iter()
                        .map(|f| {
                            let name = &f.ident.as_ref().unwrap();
                            let raw_name = name.to_string().to_snake_case();
                            quote! {
                                (HashOps::hash(&self.#name), #raw_name)
                            }
                        })
                        .collect();
                    quote! {
                        HashOpsHashable::hash(&<[_]>::into_vec(Box::new([#(#fields,)*])), hasher);
                    }
                },
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
                        use ::valence_protocol::{hash_utils::{HashOpsHashable, HashOps}, Context};

                        #encode_fields
                    }
                }
            })
        }
        Data::Enum(enum_) => {
            let variants = pair_variants_with_discriminants(enum_.variants)?;

            let encode_arms = variants
                .iter()
                .map(|(disc, variant)| {
                    let variant_name = &variant.ident;


                    match &variant.fields {
                        Fields::Named(fields) => {
                            let field_names = fields
                                .named
                                .iter()
                                .map(|f| f.ident.as_ref().unwrap())
                                .collect::<Vec<_>>();

                            let encode_fields = field_names
                                .iter()
                                .map(|name| {
                                    let raw_name = name.to_string().to_snake_case();

                                    quote! {
                                        (HashOps::hash(#name), #raw_name)
                                    }
                                })
                                .collect::<Vec<TokenStream>>();
                            let encode_fields = quote! {
                                HashOpsHashable::hash(&<[_]>::into_vec(Box::new([#(#encode_fields,)*])), hasher);
                            };

                            quote! {
                                Self::#variant_name { #(#field_names,)* } => {
                                    #encode_fields
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
                        Fields::Unit => quote! {
                            Self::#variant_name => 
                                HashOpsHashable::hash(&#disc, hasher),
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
                        use ::valence_protocol::{hash_utils::{HashOpsHashable, HashOps}, VarInt, Context};

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
