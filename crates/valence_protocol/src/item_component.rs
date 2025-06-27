use std::hash::Hasher;
use crate::hash_utils::{HashOps, HashOpsHashable};
use crate::sound::{SoundDirect, SoundId};
use crate::{Decode, Encode, GlobalPos, IDSet, VarInt};
use std::io::Write;
use valence_generated::attributes::{EntityAttribute, EntityAttributeOperation};
use valence_generated::registry_id::RegistryId;
use valence_ident::Ident;
use valence_nbt::Compound;
use valence_protocol::profile::Property;
use valence_protocol_macros::HashOps;
use valence_text::color::RgbColor;
use valence_text::Text;

include!(concat!(env!("OUT_DIR"), "/item_component.rs"));

type StrIdent = Ident<String>;

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
#[repr(i16)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl Rarity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "common" => Some(Rarity::Common),
            "uncommon" => Some(Rarity::Uncommon),
            "rare" => Some(Rarity::Rare),
            "epic" => Some(Rarity::Epic),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Epic =>  "epic"
        }
    }
}

impl HashOpsHashable for Rarity {
    fn hash<T>(&self, hasher: &mut T)
    where
        T: Hasher + Sized
    {
        HashOpsHashable::hash(&self.to_str(), hasher);
    }
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub enum ConsumeEffect {
    ApplyEffects {
        effects: Vec<PotionEffect>,
        probability: f32,
    },
    RemoveEffects {
        effects: IDSet,
    },
    ClearAllEffects,
    TeleportRandomly {
        diameter: f32,
    },
    PlaySound {
        sound: SoundDirect,
    },
}

/// Describes all the aspects of a potion effect.
// TODO: move this somewhere else
#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct PotionEffect {
    /// The ID of the effect in the potion effect type registry.
    pub id: VarInt,
    pub details: PotionEffectDetails,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct PotionEffectDetails {
    pub amplifier: VarInt,
    /// -1 for infinite.
    pub duration: VarInt,
    /// Produces more translucent particle effects if true.
    pub ambient: bool,
    /// Completely hides effect particles if false.
    pub show_particles: bool,
    /// Shows the potion icon in the inventory screen if true.
    pub show_icon: bool,
    // pub hidden_effect: Option<Box<PotionEffectDetails>>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct DamageReduction {
    pub horizontal_blocking_angle: f32,
    /// IDs in the `minecraft:damage_kind` registry.
    pub kind: Option<IDSet>,
    pub base: f32,
    pub factor: f32,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct BlockPredicate {
    pub blocks: Option<IDSet>,
    pub properties: Option<Vec<BlockPredicateProperty>>,
    pub nbt: Option<Compound>,
}

#[derive(Clone, PartialEq, Debug, HashOps)]
pub struct BlockPredicateProperty {
    pub name: String,
    pub is_exact_match: bool,
    pub exact_value: Option<String>,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
}

impl Encode for BlockPredicateProperty {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.name.encode(&mut w)?;
        self.is_exact_match.encode(&mut w)?;
        if let Some(exact_value) = &self.exact_value {
            exact_value.encode(&mut w)?;
        }
        if let Some(min_value) = &self.min_value {
            min_value.encode(&mut w)?;
        }
        if let Some(max_value) = &self.max_value {
            max_value.encode(&mut w)?;
        }
        Ok(())
    }
}

impl<'a> Decode<'a> for BlockPredicateProperty {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let name = String::decode(r)?;
        let is_exact_match = bool::decode(r)?;
        let exact_value = if is_exact_match {
            Some(String::decode(r)?)
        } else {
            None
        };
        let min_value = if !is_exact_match {
            Some(String::decode(r)?)
        } else {
            None
        };
        let max_value = if !is_exact_match {
            Some(String::decode(r)?)
        } else {
            None
        };
        Ok(BlockPredicateProperty {
            name,
            is_exact_match,
            exact_value,
            min_value,
            max_value,
        })
    }
}

// TODO: this is wrong
#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct ItemAttribute {
    pub effect: EntityAttribute,
    pub uuid: uuid::Uuid,
    pub name: String,
    pub value: f64,
    pub operation: EntityAttributeOperation,
    pub slot: AttributeSlot,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub enum AttributeSlot {
    Any,
    MainHand,
    OffHand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub enum EquipSlot {
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Offhand,
    Body,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, HashOps)]
pub enum MapPostProcessingType {
    Lock,
    Expand,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, HashOps)]
pub enum ConsumableAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Spear,
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct ToolRule {
    pub blocks: IDSet,
    pub speed: Option<f32>,
    pub correct_drop_for_blocks: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
#[allow(private_bounds)]
pub struct RawFilteredPair<V>
where
    V: Clone + PartialEq + Encode + HashOpsHashable
{
    pub raw: V,
    pub filtered: Option<V>
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub enum FireworkShape {
    SmallBall,
    LargeBall,
    STAR,
    CREEPER,
    BURST,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct FireworkExplosion {
    pub shape: FireworkShape,
    pub colors: Vec<VarInt>,
    pub fade_colors: Vec<VarInt>,
    pub has_trail: bool,
    pub has_twinkle: bool,

}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
struct GameProfile {
    pub id: uuid::Uuid,
    pub name: String,
    pub properties: Vec<Property>,

}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct BannerLayer {
    pub pattern: RegistryId,
    pub color: DyeColor,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct BeeData {
    pub entity_data: Compound,
    pub tick_in_hive: VarInt,
    pub min_ticks_in_hive: VarInt,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct StewEffect {
    pub effect: RegistryId,
    pub duration: u32,

}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct Equippable {
    /// The slot type.
    pub slot: EquipSlot,
    /// The equip sound event.
    pub equip_sound: SoundId,
    /// The model identifier.
    pub model: Option<StrIdent>,
    /// The camera overlay identifier.
    pub camera_overlay: Option<StrIdent>,
    /// Whether the item has allowed entities.
    /// The allowed entities.
    pub allowed_entities: Option<IDSet>,
    /// Whether the item is dispensable.
    pub dispensable: bool,
    /// Whether the item is swappable.
    pub swappable: bool,
    /// Whether the item takes damage on hurt.
    pub damage_on_hurt: bool,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub struct Food {

    /// Non-negative.
    pub nutrition: VarInt,
    /// How much saturation will be given after consuming the item.
    pub saturation: f32,
    /// Whether the item can always be eaten, even at full hunger.
    #[hash_ops(option=false)]
    pub can_always_eat: bool,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, HashOps)]
pub enum ItemComponent {
    /// Customizable data that doesn't fit any specific component.
    /// Always a Compound Tag.
    CustomData(Compound),
    /// Maximum stack size for the item.
    /// Ranges from 1 to 99.
    MaxStackSize(VarInt),
    /// The maximum damage the item can take before breaking.
    MaxDamage(VarInt),
    /// The current damage of the item.
    Damage(VarInt),
    /// Marks the item as unbreakable.
    Unbreakable,
    /// Item's custom name. Normally shown in italic, and changeable at an
    /// anvil.
    CustomName(Text),
    /// Override for the item's default name. Shown when the item has no custom
    /// name.
    ItemName(Text),
    /// Item's model.
    /// The model identifier.
    ItemModel(StrIdent),
    /// Item's lore.
    /// The lore lines.
    Lore(Vec<Text>),
    /// Item's rarity. This affects the default color of the item's name.
    Rarity(Rarity),
    /// The enchantments of the item.
    /// The enchantments. (The ID of the enchantment in the enchantment
    /// registry, The level of the enchantment)
    Enchantments(Vec<(StrIdent, VarInt)>),
    /// List of blocks this block can be placed on when in adventure mode.
    /// The block predicates.
    CanPlaceOn(Vec<BlockPredicate>),
    /// List of blocks this item can break when in adventure mode.
    /// The block predicates.
    CanBreak(Vec<BlockPredicate>),
    /// The attribute modifiers of the item.
    AttributeModifiers {
        /// The attributes.
        attributes: Vec<ItemAttribute>,
        /// Whether the modifiers should be shown on the item's tooltip.
        show_in_tooltip: bool,
    },
    /// Value for the item predicate when using custom item models.
    CustomModelData(VarInt),
    /// Allows you to hide all or parts of the item tooltip.
    TooltipDisplay {
        /// Whether to hide the tooltip entirely.
        #[hash_ops(option=false)]
        hide_tooltip: bool,
        // The IDs of data components in the minecraft:data_component_type registry to hide.
        hidden_components: Vec<StrIdent>,
    },
    /// Accumulated anvil usage cost.
    RepairCost(VarInt),
    /// Marks the item as non-interactive on the creative inventory (the first 5
    /// rows of items).
    /// TODO: when we send this to the client it crashes !?
    CreativeSlotLock,
    /// Overrides the item glint resulted from enchantments.
    EnchantmentGlintOverride(bool),
    /// Marks the projectile as intangible (cannot be picked-up).
    /// needs to be encoded with a empty compound tag.
    IntangibleProjectile,
    /// Makes the item restore players hunger when eaten.
    Food(Food),
    /// Makes the item consumable.
    Consumable {
        /// How long it takes to consume the item.
        consume_seconds: f32,
        /// The animation type.
        animation: ConsumableAnimation,
        /// The sound event.
        sound: SoundId,
        /// Whether the item has consume particles.
        has_consume_particles: bool,
        /// The effects.
        effects: Vec<ConsumeEffect>,
    },
    /// This specifies the item produced after using the current item.
    UseRemainder {
        /// The remainder item.
        /// TODO: Fix
        remainder: i8,
    },
    /// Cooldown to apply on use of the item.
    UseCooldown {
        /// The cooldown duration in seconds.
        seconds: f32,
        /// The cooldown group identifier.
        cooldown_group: Option<StrIdent>,
    },
    /// Marks this item as damage resistant.
    DamageResistant {
        /// Tag specifying damage types the item is immune to. Not prefixed by
        /// '#'.
        types: StrIdent,
    },
    /// Alters the speed at which this item breaks certain blocks.
    Tool {
        /// The rules.
        rules: Vec<ToolRule>,
        mining_speed: f32,
        damage_per_block: VarInt,
    },
    /// Item treated as a weapon.
    Weapon {
        /// Damage per Attack.
        damage: VarInt,
        /// How long blocking will be disabled after an attack.
        disable_blocking_for_secs: f32,
    },
    /// Allows the item to be enchanted by an enchanting table.
    Enchantable {
        /// Opaque internal value controlling how expensive enchantments may be
        /// offered.
        value: VarInt,
    },
    /// Allows the item to be equipped by the player.
    Equippable(Equippable),
    /// Items that can be combined with this item in an anvil to repair it.
    Repairable {
        /// The items.
        items: IDSet,
    },
    /// Makes the item function like elytra.
    Glider,
    /// Custom textures for the item tooltip.
    TooltipStyle {
        /// The style identifier.
        style: StrIdent,
    },
    /// Makes the item function like a totem of undying.
    DeathProtection {
        /// The effects.
        effects: Vec<ConsumeEffect>,
    },
    BlocksAttacks {
        blocks_delay_seconds: f32,
        disable_cooldown_scale: f32,
        damage_reductions: Vec<DamageReduction>,
        item_damage_threshold: f32,
        item_damage_base: f32,
        item_damage_factor: f32,
        bypassed_by: Option<StrIdent>,
        block_sound: Option<SoundId>,
        disable_sound: Option<SoundId>,
    },
    /// The enchantments stored in this enchanted book.
    StoredEnchantments {
        /// The enchantments. The first element is the enchantment ID, the
        /// second is the level.
        enchantments: Vec<(RegistryId, VarInt)>,
        show_in_tooltip: bool,
    },
    /// Color of dyed leather armor.
    DyedColor {
        /// The RGB components of the color, encoded as an integer.
        color: RgbColor,
        /// Whether the armor's color should be shown on the item's tooltip.
        show_in_tooltip: bool,
    },
    /// Color of the markings on the map item model.
    MapColor {
        /// The RGB components of the color, encoded as an integer.
        color: RgbColor,
    },
    /// The ID of the map.
    MapId {
        id: VarInt,
    },
    /// Icons present on a map.
    MapDecorations {
        /// Always a Compound Tag.
        data: Compound,
    },
    /// Used internally by the client when expanding or locking a map. Display
    /// extra information on the item's tooltip when the component is present.
    MapPostProcessing {
        /// Type of post processing.
        type_: MapPostProcessingType,
    },
    /// Projectiles loaded into a charged crossbow.
    ChargedProjectiles {
        /// The projectiles.
        // FIX: projectiles: Vec<ItemStack>,
        projectiles: i32,
    },
    /// Contents of a bundle.
    BundleContents {
        /// The items.
        // FIX: items: Vec<ItemStack>,
        items: Vec<u8>,
    },
    /// Visual and effects of a potion item.
    PotionContents {
        /// The ID of the potion type in the potion registry.
        potion_id: Option<RegistryId>,
        /// The RGB components of the color, encoded as an integer.
        custom_color: Option<RgbColor>,
        /// Any custom effects the potion might have.
        custom_effects: Vec<PotionEffect>,
        /// Custom name for the potion.
        custom_name: String,
    },
    // A duration multiplier for items that also have the `minecraft:potion_contents` component.
    PotionDurationScale {
        effects_multiplier: f32,
    },
    /// Effects granted by a suspicious stew.
    SuspiciousStewEffects(Vec<StewEffect>),
    /// Content of a writable book.
    WritableBookContent {
        /// The pages.
        pages: Vec<RawFilteredPair<String>>,
    },
    /// Content of a written and signed book.
    WrittenBookContent {
        /// The raw title of the book.
        raw_title: String,
        /// Whether the title has been filtered.
        has_filtered_title: bool,
        /// The title after going through chat filters. Only present if Has
        /// Filtered Title is true.
        filtered_title: Option<String>,
        /// The author of the book.
        author: String,
        /// The generation of the book.
        generation: VarInt,
        /// Number of elements in the following array.
        number_of_pages: VarInt,
        /// The pages.
        pages: Vec<RawFilteredPair<String>>,
        /// Whether entity selectors have already been resolved.
        resolved: bool,
    },
    /// Armor's trim pattern and color.
    Trim {
        /// ID in the `minecraft:trim_material` registry, or an inline
        /// definition.
        trim_material: String,
        /// ID in the `minecraft:trim_pattern` registry, or an inline
        /// definition.
        trim_pattern: String,
        /// Whether the trim information should be shown on the item's tooltip.
        show_in_tooltip: bool,
    },
    /// State of the debug stick.
    DebugStickState {
        /// States of previously interacted blocks. Always a Compound Tag.
        data: Compound,
    },
    /// Data for the entity to be created from this item.
    EntityData {
        /// Always a Compound Tag.
        data: Compound,
    },
    /// Data of the entity contained in this bucket.
    BucketEntityData {
        /// Always a Compound Tag.
        data: Compound,
    },
    /// Data of the block entity to be created from this item.
    BlockEntityData {
        /// Always a Compound Tag.
        data: Compound,
    },
    /// The sound played when using a goat horn.
    Instrument {
        /// ID in the `minecraft:instrument` registry, or an inline definition.
        instrument: String,
    },
    /// marked as TODO on minecraft.wiki
    ProvidesTrimMaterial,
    /// Amplifier for the effect of an ominous bottle.
    OminousBottleAmplifier {
        /// Between 0 and 4.
        amplifier: VarInt,
    },
    /// The song this item will play when inserted into a jukebox.
    JukeboxPlayable {
        /// Whether the jukebox song is specified directly, or just referenced
        /// by name.
        direct_mode: bool,
        /// The name of the jukebox song in its respective registry. Only
        /// present if Direct Mode is false.
        jukebox_song_name: Option<String>,
        /// ID in the `minecraft:jukebox_song` registry. Only present if Direct
        /// Mode is true.
        jukebox_song: Option<String>,
        /// Whether the song should be shown on the item's tooltip.
        show_in_tooltip: bool,
    },
    /// marked as TODO on minecraft.wiki
    ProvidesBannerPatterns {
        /// A pattern identifier like `#minecraft:pattern_item/globe`.
        key: StrIdent,
    },
    /// The recipes this knowledge book unlocks.
    Recipes {
        /// Always a Compound Tag.
        data: Compound,
    },
    /// The lodestone this compass points to.
    LodestoneTracker {

        /// The position the compass points to. Only present if Has Global
        /// Position is true.
        target: Option<GlobalPos>,
        /// Whether the component is removed when the associated lodestone is
        /// broken.
        tracked: bool,
    },
    /// Properties of a firework star.
    FireworkExplosion(FireworkExplosion),
    /// Properties of a firework.
    Fireworks {
        /// The flight duration.
        flight_duration: VarInt,
        /// The explosions.
        explosions: Vec<FireworkExplosion>,
    },
    /// Game Profile of a player's head.
    Profile(GameProfile),
    /// Sound played by a note block when this player's head is placed on top of
    /// it.
    NoteBlockSound(String),
    /// Patterns of a banner or banner applied to a shield.
    BannerPatterns {
        /// The layers.
        layers: Vec<BannerLayer>,
    },
    /// Base color of the banner applied to a shield.
    /// The color.
    BaseColor(DyeColor),
    /// Decorations on the four sides of a pot.
    PotDecorations {
        /// The number of elements in the following array.
        number_of_decorations: VarInt,
        /// The decorations.
        decorations: Vec<VarInt>,
    },
    /// Items inside a container of any type.
    Container {
        /// The number of elements in the following array.
        number_of_items: VarInt,
        /// The items.
        // TODO: FIX: items: Vec<ItemStack>,
        items: i8,
    },
    /// State of a block.
    BlockState {
        /// The properties.
        properties: Vec<(String, String)>,
    },
    /// Bees inside a hive.
    Bees {
        /// The bees.
        bees: Vec<VarInt>,
    },
    /// Name of the necessary key to open this container.
    /// Always a String Tag.
    Lock(String),
    /// Loot table for an unopened container.
    /// Always a Compound Tag.
    ContainerLoot(Compound),
    /// Changes the sound that plays when the item breaks.
    BreakSound(SoundId),
    /// The biome variant of a villager.
    /// An ID in the `minecraft:villager_type` registry.
    VillagerVariant(VillagerType),
    /// The variant of a wolf.
    /// An ID in the `minecraft:wolf_variant` registry.
    WolfVariant(WolfVariant),
    /// The type of sounds that a wolf makes.
    /// An ID in the `minecraft:wolf_sound_variant` registry.
    WolfSoundVariant(WolfSoundVariant),
    /// The dye color of the wolf's collar.
    WolfCollar(DyeColor),
    /// The variant of a fox.
    FoxVariant {
        /// 0: red, 1: snow.
        variant: VarInt,
    },
    /// The size of a salmon.
    SalmonSize {
        /// 0: small, 1: medium, 2: large.
        size: VarInt,
    },
    /// The variant of a parrot.
    ParrotVariant {
        /// An ID in the `minecraft:parrot_type` registry.
        variant: VarInt,
    },
    /// The pattern of a tropical fish.
    TropicalFishPattern {
        /// 0: kob, 1: sunstreak, 2: snooper, 3: dasher, 4: brinely, 5: spotty,
        /// 6: flopper, 7: stripey, 8: glitter, 9: blockfish, 10: betty, 11:
        /// clayfish.
        pattern: VarInt, // TODO: maybe also enum?
    },
    /// The base color of a tropical fish.
    TropicalFishBaseColor(DyeColor),
    /// The pattern color of a tropical fish.
    TropicalFishPatternColor(DyeColor),
    /// The variant of a mooshroom.
    MooshroomVariant {
        /// 0: red, 1: brown.
        variant: VarInt,
    },
    /// The variant of a rabbit.
    RabbitVariant {
        // 0: brown, 1: white, 2: black, 3: white splotched, 4: gold, 5: salt, 6: evil.
        variant: VarInt, // TODO: enum?
    },
    /// An ID in the `minecraft:pig_variant` registry.
    PigVariant(PigVariant),
    /// The variant of a cow.
    /// An ID in the `minecraft:cow_variant` registry.
    CowVariant(CowVariant),
    /// The variant of a chicken.
    /// An ID in the `minecraft:chicken_variant` registry.
    ChickenVariant(ChickenVariant),
    /// The variant of a frog.
    /// An ID in the `minecraft:frog_variant` registry.
    FrogVariant(FrogVariant),
    /// The variant of a horse.
    HorseVariant {
        /// 0: white, 1: creamy, 2: chestnut, 3: brown, 4: black, 5: gray, 6:
        /// dark brown.
        variant: VarInt, // TODO: enum?
    },
    /// The variant of a painting.
    PaintingVariant(PaintingVariant),
    /// The variant of a llama.
    LlamaVariant {
        /// 0: creamy, 1: white, 2: brown, 3: gray.
        variant: VarInt, // TODO: enum?
    },
    /// The variant of an axolotl.
    AxolotlVariant {
        /// 0: lucy, 1: wild, 2: gold, 3: cyan, 4: blue.
        variant: VarInt, // TODO: enum?
    },
    /// The variant of a cat.
    /// An ID in the `minecraft:cat_variant` registry.
    CatVariant(CatVariant),
    /// The dye color of the cat's collar.
    CatCollar(DyeColor),
    /// The color of a sheep.
    SheepColor(DyeColor),
    /// The color of a shulker.
    ShulkerColor(DyeColor),
}


impl ItemComponent {
    pub fn hash(&self) -> i32 {
        HashOps::hash(self)
    }

    // Create a [`ItemComponent`] from a
    // [`valence_generated::item::SerItemComponent`] (which is generated by the
    // build script). fn from_serialized(serialized: SerItemComponent) -> Self {
    //     todo!()
    // }
}
