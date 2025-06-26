use std::collections::HashSet;
use std::fmt::Debug;
use crate::{Encode, IDSet, VarInt};
use crc32c::Crc32cHasher;
use std::hash::Hasher;
use valence_generated::attributes::{EntityAttribute, EntityAttributeOperation};
use valence_generated::registry_id::RegistryId;
use valence_ident::Ident;
use valence_nbt::Compound;
use valence_text::color::RgbColor;
use valence_text::Text;
use crate::id_or::IdOr;
use crate::item_component::{ConsumeEffect, DamageReduction, FireworkExplosion, ItemAttribute, PotionEffect, BlockPredicateProperty, RawFilteredPair, ToolRule, BannerLayer, BlockPredicate, StewEffect};
use crate::profile::Property;

/// Hash a value using CRC32C as defined in minecraft's `HashOps.java` file.
pub(crate) trait HashOpsHashable {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized;
}

struct HashCode(i32);

trait ToHashCode {
    fn to_hash_code(&self) -> HashCode;
}

impl ToHashCode for BlockPredicateProperty {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for PotionEffect {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for ItemAttribute {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for ConsumeEffect {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for DamageReduction {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for ToolRule {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for BannerLayer {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for FireworkExplosion {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for Property {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for Text {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for BlockPredicate {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl ToHashCode for StewEffect {
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

impl<T> ToHashCode for RawFilteredPair<T>
where 
    T: Clone + PartialEq + Encode + HashOpsHashable
{
    fn to_hash_code(&self) -> HashCode {
        HashCode(HashOps::hash(self))
    }
}

/// Basic type for bool
impl HashOpsHashable for bool {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(13);
        hasher.write_u8(if *self { 1 } else { 0 });
    }
}


/// Basic type byte
impl HashOpsHashable for i8 {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(6);
        hasher.write_i8(*self);
    }
}

/// Basic type shirt
impl HashOpsHashable for i16 {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(7);
        hasher.write_i16(*self);
    }
}

/// Basic type int
impl HashOpsHashable for i32 {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(8);
        hasher.write_i32(*self);
    }
}

/// Basic type long
impl HashOpsHashable for i64 {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(9);
        hasher.write_i64(*self)
    }
}

/// Basic type floats
impl HashOpsHashable for f32 {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(10);
        hasher.write(&self.to_le_bytes());
    }
}

/// Basic type doubles
impl HashOpsHashable for f64 {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(11);
        hasher.write(&self.to_le_bytes());
    }
}

/// Basic type string
impl HashOpsHashable for &str {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(12);
        hasher.write(&(self.chars().count() as i32).to_le_bytes());
        for c in self.chars() {
            let c = c as i16;
            hasher.write_i8(c as i8);
            hasher.write_i8((c >> 8) as i8);
        }
    }
}

/// Basic type for raw bytes
impl HashOpsHashable for Vec<u8> {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(14);
        for &byte in self {
            hasher.write_u8(byte);
        }
        hasher.write_u8(15);
    }
}

/// Basic type for list of ints
impl HashOpsHashable for Vec<i32> {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(16);
        for &value in self {
            hasher.write_i32(value);
        }
        hasher.write_u8(17);
    }
}

/// Basic type for long lists
impl HashOpsHashable for Vec<i64> {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(18);
        for &value in self {
            hasher.write_i64(value);
        }
        hasher.write_u8(19);
    }
}

/// Basic type of field maps also used for hashing structs
///
/// Structs need to be hashed as:
/// Vec<(value, field_name)>
/// 
/// As for maps:
/// Vec<(key, value)>
impl<A, B> HashOpsHashable for Vec<(A , B)>
where 
    A: HashOpsHashable,
    B: HashOpsHashable
{
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        hasher.write_u8(2);
        for (a, b) in self {
            hasher.write_i32(HashOps::hash(a));
            hasher.write_i32(HashOps::hash(b));
        }
        hasher.write_u8(3);
    }
}

/// Basic type for hashable lists
impl< V> HashOpsHashable for Vec<V>
where V: ToHashCode
{
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        hasher.write_u8(4);
        for e in self {
            hasher.write_i32(e.to_hash_code().0);
        }
        hasher.write_u8(5);
    }
}

impl HashOpsHashable for u32 {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        HashOpsHashable::hash(&(*self as i32), hasher);
    }
}

impl HashOpsHashable for String {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        self.as_str().hash(hasher);
    }
}

impl HashOpsHashable for VarInt {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        HashOpsHashable::hash(&self.0, hasher);
    }
}

impl HashOpsHashable for IDSet {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        match self {
            IDSet::NamedSet(name) => HashOpsHashable::hash(name, hasher),
            IDSet::AdHocSet(set) => HashOpsHashable::hash(set, hasher),
        }
    }
}

impl HashOpsHashable for Vec<RegistryId> {

    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        let ids = self.iter().map(|id| id.id()).collect::<Vec<i32>>();
        HashOpsHashable::hash(&ids, hasher);
    }
}

impl HashOpsHashable for Vec<VarInt> {

    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        let var_ints = self.iter().map(|var_int| var_int.0).collect::<Vec<i32>>();
        HashOpsHashable::hash(&var_ints, hasher);
    }
}

impl HashOpsHashable for RegistryId {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        HashOpsHashable::hash(&self.id(), hasher);
    }
}

impl HashOpsHashable for Compound {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized {
        todo!()
    }
}

impl HashOpsHashable for RgbColor {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized {
        HashOpsHashable::hash(&self.into_bits(), hasher);
    }
}

impl<E> HashOpsHashable for IdOr<E>
where
    E: HashOpsHashable + Encode + Clone + Debug + PartialEq
{
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        match self {
            IdOr::Id(id) => HashOpsHashable::hash(id, hasher),
            IdOr::Inline(e) => HashOpsHashable::hash(e, hasher),
        }
    }
    
}

impl <A> HashOpsHashable for Option<A>
where A: HashOpsHashable {
    fn hash<T>(&self, hasher: &mut T)
    where 
        T: Hasher + Sized
    {
        if let Some(a) = self {
            a.hash(hasher);
        }
    }
}

impl <A> HashOpsHashable for Ident<A>
where A: HashOpsHashable
{
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        self.as_ref().hash(hasher);
    }
}

impl HashOpsHashable for Text {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized,
    {
        let data = match self.as_plain() {
            None => serde_json::to_string(&self).unwrap(),
            Some(str) => str,
        };
        HashOpsHashable::hash(&data, hasher);
    }
}

impl HashOpsHashable for EntityAttribute {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        HashOpsHashable::hash(&(self.get_id() as i32), hasher);
    }
}

impl HashOpsHashable for EntityAttributeOperation {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        HashOpsHashable::hash(&(self.to_raw() as i32), hasher);
    }
}

impl HashOpsHashable for uuid::Uuid {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        HashOpsHashable::hash(&self.to_bytes_le().to_vec(), hasher);
    }
}

pub(crate) struct HashOps;

impl HashOps {
    pub fn hash<T>(data: &T) -> i32
    where
        T: HashOpsHashable {
        let mut hasher = Crc32cHasher::default();
        data.hash(&mut hasher);
        hasher.finish() as i32
    }

    pub fn empty() -> i32 {
        let mut encoder = Crc32cHasher::default();
        encoder.write(&[2, 3]);
        encoder.finish() as i32
    }
}
mod tests {
    use valence_generated::registry_id::RegistryId;
    use valence_ident::ident;

    use crate::item_component::Rarity;
    use crate::{item_component::ItemComponent, VarInt};

    #[test]
    fn test_item_component_hashing() {
        let comp = ItemComponent::MaxStackSize(VarInt(10));
        let expected_hash = -919192125;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::Damage(VarInt(25));
        let expected_hash = 1114637767;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::Unbreakable;
        let expected_hash = -982207288;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::CustomName("Custom Item".to_string().into());
        let expected_hash = -2064782911;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::ItemName("Item name".to_string().into());
        let expceted_hash = 789562212;
        assert_eq!(comp.hash(), expceted_hash);

        let comp = ItemComponent::ItemModel(ident!("model").into());
        let expected_hash = 1591847691;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::Lore(vec!["Lore line 1".into(), "Lore line 2".into()]);
        let expected_hash = 74152878;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::Rarity(Rarity::Epic);
        let expected_hash = -292715907;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::Enchantments(vec![
            (ident!("minecraft:knockback").into(), VarInt(2)), // Knockback II
            (ident!("minecraft:sharpness").into(), VarInt(1)), // Sharpness I
        ]);
        let expected_hash = -479181350;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::TooltipDisplay {
            hide_tooltip: false,
            hidden_components: vec![VarInt(12), VarInt(13)],
        };
        let expected_hash = 1684687611;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::RepairCost(VarInt(5));
        let expected_hash = 645064431;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::EnchantmentGlintOverride(false);
        let expected_hash = 828198337;
        assert_eq!(comp.hash(), expected_hash);

        let comp = ItemComponent::Food {
            nutrition: VarInt(3),
            saturation_modifier: 10.5,
            can_always_eat: true,
        };
        let expected_hash = 1668104010;
        assert_eq!(comp.hash(), expected_hash);
    }
}
