use std::any::Any;
use std::io::Write;
use std::mem;
pub use valence_generated::item::ItemKind;
pub use valence_generated::sound::Sound;
use crate::{Decode, Encode, VarInt};
use crate::item_component::ItemComponent;
use crate::hash_utils::AdvWriter;

const NUM_ITEM_COMPONENTS: usize = 96;


#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Patchable<T> {
    Default,
    Added((T, i32)),
    Removed,
    None,
}

/// A stack of items in an inventory.
#[derive(Clone, PartialEq, Debug)]
pub struct ItemStack {
    pub item: ItemKind,
    pub count: i8,
    components: [Patchable<Box<ItemComponent>>; NUM_ITEM_COMPONENTS],
}

impl Default for ItemStack {
    fn default() -> Self {
        ItemStack::EMPTY
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct HashedItemStack {
    pub item: ItemKind,
    pub count: i8,
    components: [Patchable<i32>; NUM_ITEM_COMPONENTS],
}

impl HashedItemStack {
    pub const EMPTY: Self = Self {
        item: ItemKind::Air,
        count: 0,
        components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
    };

    pub fn new (item: ItemKind, count: i8) -> Self {
        Self {
            item,
            count,
            components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
        }
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.item, ItemKind::Air) || self.count <= 0
    }
}

impl Encode for HashedItemStack {
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        // if self.is_empty() {
        //     false.encode(&mut w)
        // } else {
        //     true.encode(&mut w)?;
        //     self.item.encode(&mut w)?;

        //     Ok(())
        // }
        todo!()
    }
}

impl Decode<'_> for HashedItemStack {
    fn decode(r: &mut &'_ [u8]) -> anyhow::Result<Self> {
        let has_item = bool::decode(r)?;
        if !has_item {
            Ok(Self::EMPTY)
        } else {
            let item = ItemKind::decode(r)?;
            let item_count = VarInt::decode(r)?;

            let mut components = [Patchable::None; NUM_ITEM_COMPONENTS];

            let components_added: Vec<(VarInt, i32)> = Vec::decode(r)?;
            let components_removed: Vec<VarInt> = Vec::decode(r)?;

            for (id, hash) in components_added {
                let id = id.0 as usize;
                if id >= NUM_ITEM_COMPONENTS {
                    return Err(anyhow::anyhow!("Invalid item component ID: {}", id));
                }
                components[id] = Patchable::Added((hash, hash));
            }

            for id in components_removed {
                let id = id.0 as usize;
                if id >= NUM_ITEM_COMPONENTS {
                    return Err(anyhow::anyhow!("Invalid item component ID: {}", id));
                }
                components[id] = Patchable::Removed;
            }

            Ok(Self {
                item,
                count: item_count.0 as i8,
                components,
            })
        }
    }
}

impl ItemStack {
    pub const EMPTY: ItemStack = ItemStack {
        item: ItemKind::Air,
        count: 0,
        components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
    };

    /// Creates a new item stack with the vanilla default components for the
    /// given [`ItemKind`].
    #[must_use]
    pub fn new(item: ItemKind, count: i8) -> Self {
        let components = item.default_components();

        Self {
            item,
            count,
            components,
        }
    }

    /// Creates a new item stack without any components, please note that the client still
    /// has the default components, but in this case, the server is not aware of them
    pub const  fn with_empty_components(item: ItemKind, count: i8) -> Self {
        Self {
            item,
            count,
            components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
        }
    }

    /// Read the components of the item stack.
    pub fn components(&self) -> Vec<&ItemComponent> {
        self.components
            .iter()
            .filter_map(|component| match component {
                Patchable::Added((v, _)) => Some(&**v),
                _ => None,
            })
            .collect()
    }

    /// Returns the default components for the [`ItemKind`].
    pub fn default_components(&self) -> Vec<ItemComponent> {
        todo!("Lets to default vales a later date")
    }

    /// Attach a component to the item stack.
    pub fn insert_component(&mut self, component: ItemComponent) {
        let id = component.id() as usize;
        let hash = component.hash();
        self.components[id] =  Patchable::Added((Box::new(component), hash));
    }

    /// Remove a component from the item stack by its ID, see
    /// [`ItemComponent::id`].
    ///
    /// Returns the removed component if it was present, otherwise `None`.
    pub fn remove_component<I: Into<usize>>(&mut self, id: I) -> Option<ItemComponent> {
        let id = id.into();
        if id < NUM_ITEM_COMPONENTS {
            match mem::replace(&mut self.components[id],  Patchable::Removed) { 
                Patchable::Added((boxed, _)) => Some(*boxed),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get a specific component by its ID, see [`ItemComponent::id`].
    pub fn get_component<I: Into<usize>>(&self, id: I) -> Option<&ItemComponent> {
        let id = id.into();
        if id < NUM_ITEM_COMPONENTS {
            match &self.components[id] {
                Patchable::Added((component, _)) => Some(&**component),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get a mutable iterator over the components of the item stack.
    pub fn components_iter_mut(&mut self) -> impl Iterator<Item = &mut ItemComponent> {
        self.components
            .iter_mut()
            .filter_map(|component| match component {
                Patchable::Added((v, _)) => Some(&mut **v),
                _ => None,
            })
    }

    #[must_use]
    pub const fn with_count(mut self, count: i8) -> Self {
        self.count = count;
        self
    }

    #[must_use]
    pub const fn with_item(mut self, item: ItemKind) -> Self {
        self.item = item;
        self
    }

    #[must_use]
    pub fn with_components(mut self, components: Vec<ItemComponent>) -> Self {
        for component in components {
            self.insert_component(component);
        }
        self
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.item, ItemKind::Air) || self.count <= 0
    }
}

impl Encode for ItemStack {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        if self.is_empty() {
            VarInt(0).encode(&mut w)
        } else {
            VarInt(i32::from(self.count)).encode(&mut w)?;
            self.item.encode(&mut w)?;

            let (components_added, components_removed) = {
                let mut removed: Vec<VarInt> = Vec::new();
                let mut added = Vec::new();

                for (i, component) in self.components.iter().enumerate() {
                    match component {
                        Patchable::Added((component, _)) => added.push(component),
                        Patchable::Removed => removed.push(VarInt(i as i32)),
                        _ => {}
                    }
                }

                (added, removed)
            };

            VarInt(components_added.len() as i32).encode(&mut w)?;
            VarInt(components_removed.len() as i32).encode(&mut w)?;

            for component in components_added {
                component.encode(&mut w)?;
            }

            for component in components_removed {
                component.encode(&mut w)?;
            }

            Ok(())
        }
    }
}

impl<'a> Decode<'a> for ItemStack {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let count = VarInt::decode(r)?.0 as i8;
        if count == 0 {
            return Ok(ItemStack::EMPTY);
        };

        let item = ItemKind::decode(r)?;

        let default_components = item.default_components();

        let components_added_count = VarInt::decode(r)?.0 as usize;
        let components_removed_count = VarInt::decode(r)?.0 as usize;
        let mut components_added: Vec<ItemComponent> = Vec::with_capacity(components_added_count);
        let mut components_removed: Vec<u32> = Vec::with_capacity(components_removed_count);

        for _ in 0..components_added_count {
            let component = ItemComponent::decode(r)?;
            components_added.push(component);
        }

        for _ in 0..components_removed_count {
            let id = VarInt::decode(r)?.0 as u32;
            components_removed.push(id);
        }

        let mut components = default_components;

        for id in components_removed {
            let id = id as usize;
            if id >= NUM_ITEM_COMPONENTS {
                return Err(anyhow::anyhow!("Invalid item component ID: {}", id));
            }
            components[id] =  Patchable::Removed;
        }

        for component in components_added {
            let id = component.id() as usize;
            let hash = component.hash();
            components[id] = Patchable::Added((Box::new(component), hash));
        }

        Ok(ItemStack {
            item,
            count,
            components,
        })
    }
}

include!(concat!(env!("OUT_DIR"), "/build_item_components_defaults.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_item_stack_is_empty() {
        let air_stack = ItemStack::new(ItemKind::Air, 10);
        let less_then_one_stack = ItemStack::new(ItemKind::Stone, 0);

        assert!(air_stack.is_empty());
        assert!(less_then_one_stack.is_empty());

        assert!(ItemStack::EMPTY.is_empty());

        let not_empty_stack = ItemStack::new(ItemKind::Stone, 10);

        assert!(!not_empty_stack.is_empty());
    }
}
