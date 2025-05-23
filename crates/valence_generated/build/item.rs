use std::{collections::HashMap, fmt};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::Deserialize;
use valence_build_utils::{ident, rerun_if_changed};

#[derive(Deserialize, Clone, Debug)]
struct Item {
    id: u16,
    name: String,
    translation_key: String,
    max_stack: i8,
    max_durability: u16,
    enchantability: u8,
    fireproof: bool,
    food: Option<FoodComponent>,
    #[serde(deserialize_with = "deserialize_components_vec")]
    components: Vec<SerItemComponent>,
}

#[derive(Deserialize, Clone, Debug)]
struct FoodComponent {
    hunger: u16,
    saturation: f32,
    always_edible: bool,
    meat: bool,
    snack: bool,
    // TODO: effects
}


#[derive(Deserialize, Clone, Debug)]
enum SerItemComponent {
    #[serde(rename = "minecraft:enchantments")]
    Enchantments {
        levels: HashMap<i32, i32>,
    },
    #[serde(rename = "minecraft:lore")]
    Lore(Vec<String>),
    #[serde(rename = "minecraft:attribute_modifiers")]
    AttributeModifiers {
        modifiers: Vec<SerAttributeModifier>,
    },
    #[serde(rename = "minecraft:max_stack_size")]
    MaxStackSize(i8),
    #[serde(rename = "minecraft:repair_cost")]
    RepairCost(i32),
    #[serde(rename = "minecraft:item_model")]
    ItemModel(String),
    #[serde(rename = "minecraft:rarity")]
    Rarity(String),
    #[serde(rename = "minecraft:item_name")]
    ItemName(String),
    #[serde(rename = "minecraft:damage")]
    Damage(i32),
    #[serde(rename = "minecraft:max_damage")]
    MaxDamage(i32),
    #[serde(rename = "minecraft:repairable")]
    Repairable {
        items: String,
    },
}

fn deserialize_components_vec<'de, D>(
    deserializer: D,
) -> Result<Vec<SerItemComponent>, D::Error>
where
    D: ::serde::Deserializer<'de>,
{
    use ::serde::de::{self, MapAccess, Visitor};
    use std::fmt;

    struct ComponentsVisitor;

    impl<'de> Visitor<'de> for ComponentsVisitor {
        type Value = Vec<SerItemComponent>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map of SerItemComponent")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut components = Vec::new();
            while let Some((key, value)) = map.next_entry::<String, ::serde_json::Value>()? {
                // Try to deserialize each entry as a SerItemComponent using the rename attributes
                let json_obj = ::serde_json::json!({ key: value });
                match ::serde_json::from_value::<SerItemComponent>(json_obj) {
                    Ok(component) => components.push(component),
                    Err(_) => continue, // skip unknown keys or invalid formats
                }
            }
            Ok(components)
        }
    }

    deserializer.deserialize_map(ComponentsVisitor)
}


#[derive(Deserialize, Clone, Debug)]  
struct SerAttributeModifier {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub amount: f64,
    pub operation: String,
    pub slot: String,
}

impl ToTokens for SerAttributeModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let type_ = &self.type_;
        let id = &self.id;
        let amount = &self.amount;
        let operation = &self.operation;
        let slot = &self.slot;

        tokens.extend(quote! {
            SerAttributeModifier {
                type_: #type_,
                id: #id,
                amount: #amount,
                operation: #operation,
                slot: #slot,
            }
        });
    }
}

pub(crate) fn build() -> anyhow::Result<TokenStream> {
    rerun_if_changed(["extracted/items.json"]);

    let items = serde_json::from_str::<Vec<Item>>(include_str!("../extracted/items.json"))?;

    let item_kind_count = items.len();

    let item_kind_from_raw_id_arms = items
        .iter()
        .map(|item| {
            let id = &item.id;
            let name = ident(item.name.to_pascal_case());

            quote! {
                #id => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_raw_id_arms = items
        .iter()
        .map(|item| {
            let id = &item.id;
            let name = ident(item.name.to_pascal_case());

            quote! {
                Self::#name => #id,
            }
        })
        .collect::<TokenStream>();

    let item_kind_from_str_arms = items
        .iter()
        .map(|item| {
            let str_name = &item.name;
            let name = ident(str_name.to_pascal_case());
            quote! {
                #str_name => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_str_arms = items
        .iter()
        .map(|item| {
            let str_name = &item.name;
            let name = ident(str_name.to_pascal_case());
            quote! {
                Self::#name => #str_name,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_translation_key_arms = items
        .iter()
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let translation_key = &item.translation_key;
            quote! {
                Self::#name => #translation_key,
            }
        })
        .collect::<TokenStream>();

    let item_kind_variants = items
        .iter()
        .map(|item| ident(item.name.to_pascal_case()))
        .collect::<Vec<_>>();

    let item_kind_to_max_stack_arms = items
        .iter()
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let max_stack = item.max_stack;

            quote! {
                Self::#name => #max_stack,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_food_component_arms = items
        .iter()
        .map(|item| match &item.food {
            Some(food_component) => {
                let name = ident(item.name.to_pascal_case());
                let hunger = food_component.hunger;
                let saturation = food_component.saturation;
                let always_edible = food_component.always_edible;
                let meat = food_component.meat;
                let snack = food_component.snack;

                quote! {
                    Self::#name => Some(FoodComponent {
                        hunger: #hunger,
                        saturation: #saturation,
                        always_edible: #always_edible,
                        meat: #meat,
                        snack: #snack,
                    }
                ),
                }
            }
            None => quote! {},
        })
        .collect::<TokenStream>();

    let item_kind_to_max_durability_arms = items
        .iter()
        .filter(|item| item.max_durability != 0)
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let max_durability = item.max_durability;

            quote! {
                Self::#name => #max_durability,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_enchantability_arms = items
        .iter()
        .filter(|item| item.enchantability != 0)
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let ench = item.enchantability;

            quote! {
                Self::#name => #ench,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_fireproof_arms = items
        .iter()
        .filter(|item| item.fireproof)
        .map(|item| {
            let name = ident(item.name.to_pascal_case());

            quote! {
                Self::#name => true,
            }
        })
        .collect::<TokenStream>();

let item_kind_to_components_arms = items
    .iter()
    .map(|item| {
        let name = ident(item.name.to_pascal_case());
        let components = &item.components;

        // Collect each component's tokens into a Vec
        let components_arms = components
            .iter()
            .map(|component| match component {
                SerItemComponent::Enchantments { levels } => {
                    let levels_vec = levels.iter().map(|(k, v)| {
                        let k = *k;
                        let v = *v;
                        quote! { (#k, #v) }
                    });
                    quote! {
                        SerItemComponent::Enchantments { levels: ::std::collections::HashMap::from_iter([#(#levels_vec),*]) }
                    }
                }
                SerItemComponent::Lore(lore) => {
                    quote! {
                        SerItemComponent::Lore(vec![#(#lore),*])
                    }
                }
                SerItemComponent::AttributeModifiers { modifiers } => {
                    let modifiers_tokens = modifiers.iter().map(|m| quote! { #m }).collect::<Vec<_>>();
                    quote! {
                        SerItemComponent::AttributeModifiers { modifiers: vec![#(#modifiers_tokens),*] }
                    }
                }
                SerItemComponent::MaxStackSize(max_stack) => {
                    quote! {
                        SerItemComponent::MaxStackSize(#max_stack)
                    }
                }
                SerItemComponent::RepairCost(repair_cost) => {
                    quote! {
                        SerItemComponent::RepairCost(#repair_cost)
                    }
                }
                SerItemComponent::ItemModel(item_model) => {
                    quote! {
                        SerItemComponent::ItemModel(#item_model)
                    }
                }
                SerItemComponent::Rarity(rarity) => {
                    quote! {
                        SerItemComponent::Rarity(#rarity)
                    }
                }
                SerItemComponent::ItemName(item_name) => {
                    quote! {
                        SerItemComponent::ItemName(#item_name)
                    }
                }
                SerItemComponent::Damage(damage) => {
                    quote! {
                        SerItemComponent::Damage(#damage)
                    }
                }
                SerItemComponent::MaxDamage(max_damage) => {
                    quote! {
                        SerItemComponent::MaxDamage(#max_damage)
                    }
                }
                SerItemComponent::Repairable { items } => {
                    quote! {
                        SerItemComponent::Repairable { items: #items }
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            Self::#name => vec![#(#components_arms),*],
        }
    })
    .collect::<TokenStream>();
        

    Ok(quote! {
        use crate::registry_id::RegistryId;
        use std::collections::HashMap;
        /// Represents an item from the game
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
        #[repr(u16)]
        pub enum ItemKind {
            #[default]
            #(#item_kind_variants,)*
        }

        /// Contains food information about an item.
        ///
        /// Only food items have a food component.
        #[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
        pub struct FoodComponent {
            pub hunger: u16,
            pub saturation: f32,
            pub always_edible: bool,
            pub meat: bool,
            pub snack: bool,
        }

        /// Contains the serializable components of an item.
        #[derive(Clone, PartialEq, Debug)]
        pub enum SerItemComponent {
            Enchantments {
                levels: HashMap<i32, i32>,
            },
            Lore(Vec<&'static str>),
            AttributeModifiers {
                modifiers: Vec<SerAttributeModifier>,
            },
            MaxStackSize(i8),
            RepairCost(i32),
            ItemModel(&'static str),
            Rarity(&'static str),
            ItemName(&'static str),
            Damage(i32),
            MaxDamage(i32),
            Repairable {
                items: &'static str,
            },
        }
            
        #[derive(Clone, PartialEq, Debug)]  
        pub struct SerAttributeModifier {
            pub type_: &'static str,
            pub id: &'static str,
            pub amount: f64,
            pub operation: &'static str,
            pub slot: &'static str,
        }

        impl ItemKind {
            /// Constructs a item kind from a raw item ID.
            ///
            /// If the given ID is invalid, `None` is returned.
            pub const fn from_raw(id: u16) -> Option<Self> {
                match id {
                    #item_kind_from_raw_id_arms
                    _ => None
                }
            }

            /// Gets the raw item ID from the item kind
            pub const fn to_raw(self) -> u16 {
                match self {
                    #item_kind_to_raw_id_arms
                }
            }

            /// Construct an item kind for its snake_case name.
            ///
            /// Returns `None` if the name is invalid.
            #[allow(clippy::should_implement_trait)]
            pub fn from_str(name: &str) -> Option<ItemKind> {
                match name {
                    #item_kind_from_str_arms
                    _ => None
                }
            }

            /// Gets the snake_case name of this item kind.
            pub const fn to_str(self) -> &'static str {
                match self {
                    #item_kind_to_str_arms
                }
            }

            /// Gets the translation key of this item kind.
            pub const fn translation_key(self) -> &'static str {
                match self {
                    #item_kind_to_translation_key_arms
                }
            }

            /// Returns the maximum stack count.
            pub const fn max_stack(self) -> i8 {
                match self {
                    #item_kind_to_max_stack_arms
                }
            }

            /// Returns a food component which stores hunger, saturation etc.
            ///
            /// If the item kind can't be eaten, `None` will be returned.
            pub const fn food_component(self) -> Option<FoodComponent> {
                match self {
                    #item_kind_to_food_component_arms
                    _ => None
                }
            }

            /// Returns the maximum durability before the item will break.
            ///
            /// If the item doesn't have durability, `0` is returned.
            pub const fn max_durability(self) -> u16 {
                match self {
                    #item_kind_to_max_durability_arms
                    _ => 0,
                }
            }

            /// Returns the enchantability of the item kind.
            ///
            /// If the item doesn't have durability, `0` is returned.
            pub const fn enchantability(self) -> u8 {
                match self {
                    #item_kind_to_enchantability_arms
                    _ => 0,
                }
            }

            /// Returns if the item can survive in fire/lava.
            pub const fn fireproof(self) -> bool {
                #[allow(clippy::match_like_matches_macro)]
                match self {
                    #item_kind_to_fireproof_arms
                    _ => false
                }
            }

            /// Gets the Serializable default components of this item kind.
            // TODO: make this constant maybe?
            pub fn ser_components(self) -> Vec<SerItemComponent> {
                match self {
                    #item_kind_to_components_arms
                    _ => unreachable!()
                }
            }


            /*
            /// Constructs an item kind from a block kind.
            ///
            /// [`ItemKind::Air`] is used to indicate the absence of an item.
            pub const fn from_block_kind(kind: BlockKind) -> Self {
                kind.to_item_kind()
            }

            /// Constructs a block kind from an item kind.
            ///
            /// If the given item kind doesn't have a corresponding block kind, `None` is returned.
            pub const fn to_block_kind(self) -> Option<BlockKind> {
                BlockKind::from_item_kind(self)
            }*/

            /// An array of all item kinds.
            pub const ALL: [Self; #item_kind_count] = [#(Self::#item_kind_variants,)*];
        }

        impl From<ItemKind> for RegistryId {
            fn from(item: ItemKind) -> Self {
                RegistryId::new(item.to_raw() as i32)
            }
        }
    })
}
