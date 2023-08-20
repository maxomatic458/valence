use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::Expr::Field;
use syn::{
    parenthesized, parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, LitStr, Meta,
    MetaNameValue, Path,
};

// pub trait Command {
//     type CommandExecutables: Send + Sync; // usually an enum of all the possible commands
//
//     fn name() -> String;
//     fn assemble_graph(&self, graph: &mut CommandGraphBuilder<Self::CommandExecutables>);
// }

// #[derive(Command)]
// #[paths = ["selectfruit", "select fruit", "sf"]]
// #[scopes = ["valence:command:teleport"]]
// enum SelectFruit {
//     #[paths = "apple"] // this path is from the perant: selectfruit so `/selectfruit apple` will be here
//     Apple,
//     #[paths = "banana"]
//     Banana,
//     #[paths = "Strawberry {0?}"] // this could be `/selectfruit banana green` or /selectfruit banana
//     // the macro should be able to detect the fact it is optional and register two executables;
//     // one has no args and the other has the optional arg
//     Strawberry(Option<Strawberry>),
// }
//
// #[derive(Suggestions)] // I'd want this to assume snake case unless manully set
// enum Strawberry {
//     Red,
//     Green
// }

enum CommandBranch {
    Branch(Path),
    SingleArg(String, Path, Ident),
}

#[proc_macro_derive(Command, attributes(command, scopes, paths))]
pub fn derive_command(a_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(a_input as DeriveInput);

    let enum_name = input.ident;

    let mut alias_paths = input
        .attrs
        .iter()
        .filter_map(parse_path)// get_command_attr returns Option<(String, Vec<String>)>
        .next()// there should only be one command name
        .expect("Command names not provided");

    let base_path = alias_paths.remove(0);

    let outer_scopes = input
        .attrs
        .iter()
        .filter_map(|attr| get_lit_list_attr(attr, "scopes"))
        .next()
        .unwrap_or(Vec::new());

    let fields = match input.data {
        Data::Enum(ref data_enum) => &data_enum.variants,
        _ => panic!("Command must be an enum"),
    };

    let mut paths = Vec::new();
    // let mut expanded_variants = Vec::new();
    for variant in fields {
        for attr in variant.attrs.iter() {
            if let Some(attr_paths) = parse_path(attr) {
                paths.push((attr_paths, variant.fields.clone(), variant.ident.clone()));
            }
        }
        // let mut expanded_variant_fields = Vec::new();
        // match variant.fields {
        //     Fields::Named(ref fields) => {
        //         for field in fields.named.iter() {
        //             let field_ident = field.ident.as_ref().unwrap();
        //             let field_type = &field.ty;
        //
        //             let mut inner;
        //             let mut is_optional = false;
        //
        //             match field_type {
        //                 syn::Type::Path(ref type_path) => {
        //                     let path = &type_path.path;
        //                     if path.segments.len() != 1 {
        //                         inner = field_type;
        //                     }
        //                     let segment = &path.segments.first().unwrap();
        //                     if segment.ident.to_string() != "Option" {
        //                         inner = field_type
        //                     }
        //                     match &segment.arguments {
        //                         syn::PathArguments::AngleBracketed(ref angle_bracketed) => {
        //                             if angle_bracketed.args.len() != 1 {
        //                                 inner = field_type;
        //                             }
        //                             match angle_bracketed.args.first().unwrap() {
        //                                 syn::GenericArgument::Type(ref generic_type) => {
        //                                     inner = generic_type;
        //                                     is_optional = true;
        //                                 }
        //                                 _ => inner = field_type,
        //                             }
        //                         }
        //                         _ => inner = field_type,
        //                     }
        //                 }
        //                 _ => inner = field_type,
        //             };
        //
        //             if !is_optional {
        //                 expanded_variant_fields.push(quote! {
        //                 #field_ident: #inner
        //             });
        //             } else {
        //                 expanded_variant_fields.push(quote! {
        //                 #field_ident: Option<<#inner as valence_command::arg_parser::CommandArg>::Result>
        //             });
        //             }
        //         }
        //     }
        //     _ => panic!("Command enum variants must be named"),
        // }
        // let variant_ident = &variant.ident;
        // expanded_variants.push(quote! {
        //     #variant_ident{#(#expanded_variant_fields),*}
        // });
    }
    println!("paths: {:#?}", paths);

    let mut expanded_nodes = Vec::new();

    for (paths, fields, variant_ident) in paths {
        expanded_nodes.push({
            let processed = process_paths(&enum_name, paths, &fields, variant_ident.clone(), true);
            quote! { #processed; }
        });
    }

    let base_command_expansion = {
        let processed = process_paths(
            &enum_name,
            vec![base_path],
            &Fields::Unit,
            format_ident!("{}Root", enum_name),
            false,
        ); // this will error if the base path has args
        let mut expanded_main_command = quote! {
            let command_root_node = #processed
        };

        if !outer_scopes.is_empty() {
            expanded_main_command = quote! {
                #expanded_main_command
                    .with_scopes(vec![#(#outer_scopes),*])
            }
        }

        quote! {
            #expanded_main_command.id();
        }
    };

    let command_alias_expansion = {
        let mut alias_expansion = quote! {};
        for path in alias_paths {
            let processed = process_paths(
                &enum_name,
                vec![path],
                &Fields::Unit,
                format_ident!("{}Root", enum_name),
                false,
            );

            alias_expansion = quote! {
                #alias_expansion

                #processed
                    .redirect_to(command_root_node)
            };

            if !outer_scopes.is_empty() {
                alias_expansion = quote! {
                    #alias_expansion
                        .with_scopes(vec![#(#outer_scopes),*])
                }
            }

            alias_expansion = quote! {
                #alias_expansion;
            }
        }

        alias_expansion
    };

    println!(
        "expanded_nodes: {}",
        quote! {
            #(#expanded_nodes)*
        }
    );

    let new_struct = format_ident!("{}Command", enum_name);

    let expanded = quote! {

        impl valence_command::Command for #enum_name {
            fn assemble_graph(command_graph: &mut valence_command::command_graph::CommandGraphBuilder<Self>) {
                #base_command_expansion

                #command_alias_expansion

                #(#expanded_nodes)*
            }
        }
    };

    println!("expanded: {}", expanded);

    proc_macro::TokenStream::from(expanded)
}

fn process_paths(
    enum_name: &Ident,
    paths: Vec<Vec<CommandArg>>,
    fields: &Fields,
    variant_ident: Ident,
    executables: bool,
) -> proc_macro2::TokenStream {
    let mut inner_expansion = quote! {};
    let mut first = true;

    for path in paths {
        if !first {
            inner_expansion = if executables {
                quote! {
                        #inner_expansion;

                        command_graph.at(command_root_node)
                }
            } else {
                quote! {
                    #inner_expansion;

                    command_graph.root()
                }
            };
        } else {
            inner_expansion = if executables {
                quote! {
                    command_graph.at(command_root_node)
                }
            } else {
                quote! {
                    command_graph.root()
                }
            };

            first = false;
        }

        let mut real_args: usize = 0;
        let mut final_executable = Vec::new();
        for (i, arg) in path.iter().enumerate() {
            match arg {
                CommandArg::Literal(lit) => {
                    println!("lit: {:?}", lit);

                    inner_expansion = quote! {
                        #inner_expansion.literal(#lit)

                    };
                    if executables && i == path.len() - 1 {
                        inner_expansion = quote! {
                            #inner_expansion
                                .with_executable(|s| #enum_name::#variant_ident{#(#final_executable,)*})
                        };
                    }
                }
                CommandArg::Required(ident) => {
                    let field_type = &fields
                        .iter()
                        .find(|field| field.ident.as_ref().unwrap() == ident)
                        .expect("Required arg not found")
                        .ty;
                    let ident_string = ident.to_string();

                    inner_expansion = quote! {
                        #inner_expansion
                            .argument(#ident_string)
                            .with_parser::<#field_type>()
                    };

                    final_executable.push(quote! {
                        #ident: #field_type::parse_arg(s).unwrap()
                    });

                    if i == path.len() - 1 {
                        inner_expansion = quote! {
                            #inner_expansion
                                .with_executable(|s| {
                                    #enum_name::#variant_ident {
                                        #(#final_executable,)*
                                    }
                                })
                        };
                    }

                    real_args += 1;
                }
                CommandArg::Optional(ident) => {
                    let field_type = &fields
                        .iter()
                        .find(|field| field.ident.as_ref().unwrap() == ident)
                        .expect("Optional arg not found")
                        .ty;
                    let so_far_ident = format_ident!("graph_til_{}", ident);

                    // get what is inside the Option<...>
                    let option_inner = match field_type {
                        syn::Type::Path(ref type_path) => {
                            let path = &type_path.path;
                            if path.segments.len() != 1 {
                                panic!("Option type must be a single path segment");
                            }
                            let segment = &path.segments.first().unwrap();
                            if segment.ident.to_string() != "Option" {
                                panic!("Must be an Option type");
                            }
                            match &segment.arguments {
                                syn::PathArguments::AngleBracketed(ref angle_bracketed) => {
                                    if angle_bracketed.args.len() != 1 {
                                        panic!("Option type must have a single generic argument");
                                    }
                                    match angle_bracketed.args.first().unwrap() {
                                        syn::GenericArgument::Type(ref generic_type) => {
                                            generic_type
                                        }
                                        _ => panic!(
                                            "Option type must have a single generic argument"
                                        ),
                                    }
                                }
                                _ => panic!("Option type must have a single generic argument"),
                            }
                        }
                        _ => panic!("Option type must be a single path segment"),
                    };

                    let ident_string = ident.to_string();

                    // find the ident of all following optional args
                    let mut next_optional_args = Vec::new();
                    for next_arg in path.iter().skip(i + 1) {
                        match next_arg {
                            CommandArg::Optional(ident) => next_optional_args.push(ident),
                            _ => panic!(
                                "Only optional args can follow an optional arg, found {:?}",
                                next_arg
                            ),
                        }
                    }

                    inner_expansion = quote! {
                        let #so_far_ident = {#inner_expansion
                            .with_executable(|s| {
                                #enum_name::#variant_ident {
                                    #(#final_executable,)*
                                    #ident: None,
                                    #(#next_optional_args: None,)*
                                }
                            })
                            .id()};

                        command_graph.at(#so_far_ident)
                            .argument(#ident_string)
                            .with_parser::<#option_inner>()
                    };

                    final_executable.push(quote! {
                        #ident: Some(#option_inner::parse_arg(s).unwrap())
                    });

                    if i == path.len() - 1 {
                        inner_expansion = quote! {
                            #inner_expansion
                                .with_executable(|s| {
                                    #enum_name::#variant_ident {
                                        #(#final_executable,)*
                                    }
                                })
                        };
                    }

                    real_args += 1;
                }
            }
        }
    }
    quote!(#inner_expansion)
}

#[derive(Debug)]
enum CommandArg {
    Required(Ident),
    Optional(Ident),
    Literal(String),
}

// example input: #[paths = "strawberry {0?}"]
// example output: [CommandArg::Literal("Strawberry"), CommandArg::Optional(0)]
fn parse_path(path: &Attribute) -> Option<Vec<Vec<CommandArg>>> {
    println!("path: {:#?}", path);
    let path_strings: Vec<String> = get_lit_list_attr(path, "paths")?;

    let mut paths = Vec::new();
    // we now have the path as a string eg "strawberry {0?}"
    // the first word is a literal
    // the next word is an optional arg with the index 0
    for path_str in path_strings {
        let mut args = Vec::new();
        for word in path_str.split_whitespace() {
            if word.starts_with('{') && word.ends_with('}') {
                if word.ends_with("?}") {
                    args.push(CommandArg::Optional(format_ident!(
                        "{}",
                        word[1..word.len() - 2].to_string()
                    )));
                } else {
                    args.push(CommandArg::Required(format_ident!(
                        "{}",
                        word[1..word.len() - 1].to_string()
                    )));
                }
            } else {
                println!("making literal: {:?}", word);
                args.push(CommandArg::Literal(word.to_string()));
            }
        }
        paths.push(args);
    }

    Some(paths)
}

fn get_lit_list_attr(attr: &Attribute, ident: &str) -> Option<Vec<String>> {
    match attr.meta {
        Meta::NameValue(ref key_value) => {
            if !key_value.path.is_ident(ident) {
                return None;
            }

            match key_value.value {
                Expr::Lit(ref lit) => match lit.lit {
                    syn::Lit::Str(ref lit_str) => Some(vec![lit_str.value()]),
                    _ => return None,
                },
                _ => return None,
            }
        }
        Meta::List(ref list) => {
            if !list.path.is_ident(ident) {
                return None;
            }

            let mut path_strings = Vec::new();
            println!("list: {:#?}", list.tokens);
            // parse as array with strings
            let mut comma_next = false;
            for token in list.tokens.clone() {
                match token {
                    TokenTree::Literal(lit) => {
                        println!("lit: {:#?}", lit);
                        if comma_next {
                            return None;
                        }
                        let lit_str = lit.to_string();
                        path_strings.push(
                            lit_str
                                .strip_prefix('"')
                                .unwrap()
                                .strip_suffix('"')
                                .unwrap()
                                .to_string(),
                        );
                        comma_next = true;
                    }
                    TokenTree::Punct(punct) => {
                        if punct.as_char() != ',' || !comma_next {
                            return None;
                        }
                        comma_next = false;
                    }
                    _ => return None,
                }
            }
            Some(path_strings)
        }
        _ => return None,
    }
}
//
// fn get_scopes_attr(attr: &Attribute) -> Option<Vec<String>> {
//     // Parse args (which contain key & value).
//     // println!("attr: {:#?}", attr);
//     let key_value: MetaNameValue = match attr.meta {
//         Meta::NameValue(ref key_value) => key_value.clone(),
//         _ => return None,
//     };
//     // println!("key_value: {:#?}", key_value);
//
//     if key_value.path.is_ident("scopes") {
//         let value = match key_value.value {
//             Expr::Array(ref array) => {
//                 let mut values = Vec::new();
//                 for expr in array.elems.iter() {
//                     if let Expr::Lit(ref lit) = expr {
//                         if let syn::Lit::Str(ref lit_str) = lit.lit {
//                             values.push(lit_str.value());
//                         }
//                     } else {
//                         panic!("Command name must be an array of strings");
//                     }
//                 }
//                 values
//             }
//             _ => panic!("Command name must be an array of strings"),
//         };
//         println!("value: {:#?}", value);
//         return Some(value);
//     }
//
//     None
// }

fn get_arg_attr(attr: &Attribute) -> Option<String> {
    // Parse args (which contain key & value).
    // println!("attr: {:#?}", attr);
    let key_value: MetaNameValue = match attr.meta {
        Meta::NameValue(ref key_value) => key_value.clone(),
        _ => return None,
    };
    // println!("key_value: {:#?}", key_value);

    if key_value.path.is_ident("arg") {
        let value = match key_value.value {
            Expr::Lit(ref lit) => {
                if let syn::Lit::Str(ref lit_str) = lit.lit {
                    lit_str.value()
                } else {
                    panic!("Arg must be a string");
                }
            }
            _ => panic!("Arg must be a string"),
        };
        println!("value: {:#?}", value);
        return Some(value);
    }

    None
}
