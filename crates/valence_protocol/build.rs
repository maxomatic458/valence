use std::collections::BTreeMap;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use serde_json::Value;
use valence_build_utils::{ident, rerun_if_changed, write_generated_file};

#[derive(Clone, Debug, Deserialize)]
struct Misc {
    component_data_type: BTreeMap<String, u32>,
    villager_type: BTreeMap<String, u32>,
    cat_variant: BTreeMap<String, u32>,
    frog_variant: BTreeMap<String, u32>,
    wolf_variant: BTreeMap<String, u32>,
    pig_variant: BTreeMap<String, u32>,
    cow_variant: BTreeMap<String, u32>,
    chicken_variant: BTreeMap<String, u32>,
    painting_variant: BTreeMap<String, u32>,
    wolf_sound_variant: BTreeMap<String, u32>,
    particle_type: BTreeMap<String, u32>,
    dye_color: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Deserialize)]
struct Item {
    name: String,
    components: BTreeMap<String, Value>,
}

pub fn main() -> anyhow::Result<()> {
    rerun_if_changed(["extracted/misc.json", "extracted/items.json"]);

    write_generated_file(build_item_components()?, "item_component.rs")?;
    write_generated_file(build_item_components_defaults()?, "build_item_components_defaults.rs")?;
    Ok(())
}

fn to_sorted_vec<K, V>(data: &BTreeMap<K, V>) -> Vec<(&K, &V)>
where
V: Ord {
    let mut result: Vec<(&K, &V)> = data.into_iter()
    .map(|(k, v)| (k, v))
    .collect();
    result.sort_by(|(_, id1), (_, id2)| id1.cmp(id2));
    result

}

fn to_enum_token_steam(name: &str, data: &BTreeMap<String, u32>) -> TokenStream {
    let name = ident(name.to_pascal_case());
    let data =  to_sorted_vec(data).iter().map(|(k, v)| {
        let name = ident(k.to_pascal_case());
        (name, *v)
    }).collect::<Vec<_>>();
    let names = data.iter()
        .map(|(k, _)| quote! { #k, })
        .collect::<TokenStream>();

    quote! {
        #[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode, HashOps)]
        pub enum #name {
            #names
        }
    }
}

fn build_item_components_defaults() -> anyhow::Result<TokenStream> {
    let items: Vec<Item> = serde_json::from_str(include_str!("extracted/items.json"))?;
    let misc: Misc = serde_json::from_str(include_str!("extracted/misc.json"))?;

    let component_ids = to_sorted_vec(&misc.component_data_type).iter()
        .map(|(k, v)| {
            let name = ident(k
                .strip_prefix("minecraft:").unwrap()
                .replace("/", "_")
                .to_pascal_case());
            quote!(
            ItemComponent::#name { .. } => #v,
        )
        }).collect::<TokenStream>();

    let default_components = items.iter().map(|item| {
        let mut defaults = item.components.keys()
            .map(|k| {
                let id = *misc.component_data_type.get(k)
                    .expect("No id found for datacomponent");
                (k, id)
            })
            .collect::<Vec<_>>();
        defaults.sort_by(|(_, id1), (_, id2)| id1.cmp(id2));
        let default_comment: String = defaults.iter()
            .map(|(k, _)| k.strip_prefix("minecraft:").unwrap())
            .collect::<Vec<&str>>()
            .join(", ");
        let default_ids = defaults.iter()
            .map(|(_, id)| quote!(#id, ))
            .collect::<TokenStream>();
        let item = ident(item.name.to_pascal_case());

        quote! {
           #[doc = #default_comment]
           ItemKind::#item => vec![#default_ids],
       }
    }).collect::<TokenStream>();


    Ok(quote! {
        impl ItemComponent {
            pub fn id(&self) -> u32 {
                match self {
                    #component_ids
                }
            }
        }

        pub trait ItemKindExt {
            /// Returns the default components for the [`ItemKind`].
            fn default_components(&self) -> [Patchable<Box<ItemComponent>>; NUM_ITEM_COMPONENTS];
        }

        impl ItemKindExt for ItemKind {
            #![allow(unused_doc_comments)]
            fn default_components(&self) -> [Patchable<Box<ItemComponent>>; NUM_ITEM_COMPONENTS] {
                let mut result = [const { Patchable::None }; NUM_ITEM_COMPONENTS];
                let defaults = match self {
                    #default_components
                };
                for id in defaults {
                    result[id as usize] = Patchable::Default;
                }
                result
            }
        }
    })
}

fn build_item_components() -> anyhow::Result<TokenStream> {
    let misc: Misc = serde_json::from_str(include_str!("extracted/misc.json"))?;
    let variants = vec![
        // to_enum_token_steam("entity_animation", &misc.entity_animation),
        to_enum_token_steam("villager_type", &misc.villager_type),
        // to_enum_token_steam("villager_profession", &misc.villager_profession),
        to_enum_token_steam("cat_variant", &misc.cat_variant),
        to_enum_token_steam("frog_variant", &misc.frog_variant),
        to_enum_token_steam("wolf_variant", &misc.wolf_variant),
        to_enum_token_steam("pig_variant", &misc.pig_variant),
        to_enum_token_steam("cow_variant", &misc.cow_variant),
        to_enum_token_steam("chicken_variant", &misc.chicken_variant),
        to_enum_token_steam("painting_variant", &misc.painting_variant),
        to_enum_token_steam("wolf_sound_variant", &misc.wolf_sound_variant),
        // to_enum_token_steam("entity_pose", &misc.entity_pose),
        // to_enum_token_steam("sniffer_state", &misc.sniffer_state),
        // to_enum_token_steam("armadillo_state", &misc.armadillo_state),
        to_enum_token_steam("dye_color", &misc.dye_color),
    ].into_iter().collect::<TokenStream>();

    Ok(quote! {
        #variants
    })
}
