use serde::{Deserialize, Serialize};
use valence_server::{nbt::Compound, ItemKind, ItemStack};

/// A Wrapper around [`ItemKind`] that provides serialization and deserialization support.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct SerItemKind(pub ItemKind);

impl Serialize for SerItemKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerItemKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let item_kind = ItemKind::from_str(s).ok_or_else(|| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &"the snake case item name, like \"white_wool\"")
        });

        item_kind.map(SerItemKind)
    }
}

impl From<ItemKind> for SerItemKind {
    fn from(kind: ItemKind) -> Self {
        SerItemKind(kind)
    }
}

impl From<SerItemKind> for ItemKind {
    fn from(kind: SerItemKind) -> Self {
        kind.0
    }
}

/// A Wrapper around [`ItemStack`] that provides serialization and deserialization support.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SerItemStack {
    pub item: SerItemKind,
    pub count: i8,
    pub nbt: Option<Compound>,
}

impl From<ItemStack> for SerItemStack {
    fn from(stack: ItemStack) -> Self {
        SerItemStack {
            item: SerItemKind(stack.item),
            count: stack.count,
            nbt: stack.nbt,
        }
    }
}

impl From<SerItemStack> for ItemStack {
    fn from(stack: SerItemStack) -> Self {
        ItemStack {
            item: stack.item.0,
            count: stack.count,
            nbt: stack.nbt,
        }
    }
}

