package rs.valence.extractor.extractors;

import com.google.gson.*;

import com.mojang.authlib.GameProfile;
import net.minecraft.block.BlockState;
import net.minecraft.entity.Entity;
import net.minecraft.entity.EntityPose;
import net.minecraft.entity.EntityType;
import net.minecraft.entity.LivingEntity;
import net.minecraft.entity.attribute.DefaultAttributeRegistry;
import net.minecraft.entity.attribute.EntityAttribute;
import net.minecraft.entity.attribute.EntityAttributeInstance;
import net.minecraft.entity.data.DataTracker;
import net.minecraft.entity.data.TrackedData;
import net.minecraft.entity.data.TrackedDataHandlerRegistry;
import net.minecraft.entity.passive.*;
import net.minecraft.item.ItemStack;
import net.minecraft.particle.ParticleEffect;
import net.minecraft.registry.Registries;
import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.text.Text;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.EulerAngle;
import net.minecraft.util.math.GlobalPos;
import net.minecraft.village.VillagerData;
import net.minecraft.world.World;
import org.jetbrains.annotations.Nullable;
import org.joml.Quaternionf;
import org.joml.Vector3f;

import rs.valence.extractor.ClassComparator;
import rs.valence.extractor.DummyPlayerEntity;
import rs.valence.extractor.DummyWorld;
import rs.valence.extractor.Main;

import java.lang.reflect.ParameterizedType;
import java.util.*;

public class Entities implements Main.Extractor {
    private final ServerWorld world;

    public Entities(final MinecraftServer server) {
        world = server.getOverworld();
    }

    private static Main.Pair<String, JsonElement> trackedDataToJson(final TrackedData<?> data, final DataTracker tracker) {
        var handler = data.dataType();
        var val = tracker.get(data);

        if (handler == TrackedDataHandlerRegistry.BYTE) {
            return new Main.Pair<>("byte", new JsonPrimitive((Byte) val));
        } else if (handler == TrackedDataHandlerRegistry.INTEGER) {
            return new Main.Pair<>("integer", new JsonPrimitive((Integer) val));
        } else if (handler == TrackedDataHandlerRegistry.LONG) {
            return new Main.Pair<>("long", new JsonPrimitive((Long) val));
        } else if (handler == TrackedDataHandlerRegistry.FLOAT) {
            return new Main.Pair<>("float", new JsonPrimitive((Float) val));
        } else if (handler == TrackedDataHandlerRegistry.STRING) {
            return new Main.Pair<>("string", new JsonPrimitive((String) val));
        } else if (handler == TrackedDataHandlerRegistry.TEXT_COMPONENT) {
            // TODO: return text as json element.
            return new Main.Pair<>("text_component", new JsonPrimitive(((Text) val).getString()));
        } else if (handler == TrackedDataHandlerRegistry.OPTIONAL_TEXT_COMPONENT) {
            final var res = ((Optional<?>) val).map(o -> (JsonElement) new JsonPrimitive(((Text) o).getString()))
                    .orElse(JsonNull.INSTANCE);
            return new Main.Pair<>("optional_text_component", res);
        } else if (handler == TrackedDataHandlerRegistry.ITEM_STACK) {
            // TODO
            return new Main.Pair<>("item_stack", new JsonPrimitive(((ItemStack) val).toString()));
        } else if (handler == TrackedDataHandlerRegistry.BOOLEAN) {
            return new Main.Pair<>("boolean", new JsonPrimitive((Boolean) val));
        } else if (handler == TrackedDataHandlerRegistry.ROTATION) {
            final var json = new JsonObject();
            final var ea = (EulerAngle) val;
            json.addProperty("pitch", ea.getPitch());
            json.addProperty("yaw", ea.getYaw());
            json.addProperty("roll", ea.getRoll());
            return new Main.Pair<>("rotation", json);
        } else if (handler == TrackedDataHandlerRegistry.BLOCK_POS) {
            final var bp = (BlockPos) val;
            final var json = new JsonObject();
            json.addProperty("x", bp.getX());
            json.addProperty("y", bp.getY());
            json.addProperty("z", bp.getZ());
            return new Main.Pair<>("block_pos", json);
        } else if (handler == TrackedDataHandlerRegistry.OPTIONAL_BLOCK_POS) {
            return new Main.Pair<>("optional_block_pos", ((Optional<?>) val).map(o -> {
                final var bp = (BlockPos) o;
                final var json = new JsonObject();
                json.addProperty("x", bp.getX());
                json.addProperty("y", bp.getY());
                json.addProperty("z", bp.getZ());
                return (JsonElement) json;
            }).orElse(JsonNull.INSTANCE));
        } else if (handler == TrackedDataHandlerRegistry.FACING) {
            return new Main.Pair<>("facing", new JsonPrimitive(val.toString()));
        } else if (handler == TrackedDataHandlerRegistry.OPTIONAL_UUID) {
            final var res = ((Optional<?>) val).map(o -> (JsonElement) new JsonPrimitive(o.toString()))
                    .orElse(JsonNull.INSTANCE);
            return new Main.Pair<>("optional_uuid", res);
        } else if (handler == TrackedDataHandlerRegistry.BLOCK_STATE) {
            // TODO: get raw block state ID.
            final var state = (BlockState) val;
            return new Main.Pair<>("block_state", new JsonPrimitive(state.toString()));
        } else if (handler == TrackedDataHandlerRegistry.OPTIONAL_BLOCK_STATE) {
            // TODO: get raw block state ID.
            final var res = ((Optional<?>) val).map(o -> (JsonElement) new JsonPrimitive(o.toString()))
                    .orElse(JsonNull.INSTANCE);
            return new Main.Pair<>("optional_block_state", res);
        } else if (handler == TrackedDataHandlerRegistry.NBT_COMPOUND) {
            // TODO: base64 binary representation or SNBT?
            return new Main.Pair<>("nbt_compound", new JsonPrimitive(val.toString()));
        } else if (handler == TrackedDataHandlerRegistry.PARTICLE) {
            final var id = Registries.PARTICLE_TYPE.getId(((ParticleEffect) val).getType());
            return new Main.Pair<>("particle", new JsonPrimitive(id.getPath()));
        } else if (handler == TrackedDataHandlerRegistry.PARTICLE_LIST) {
            final List<ParticleEffect> particleList = (List<ParticleEffect>) val;
            final JsonArray json = new JsonArray();
            for (final ParticleEffect particleEffect : particleList) {
                final var id = Registries.PARTICLE_TYPE.getId(((ParticleEffect) val).getType());
                json.add(id.getPath());
            }
            return new Main.Pair<>("particle_list", json);
        } else if (handler == TrackedDataHandlerRegistry.VILLAGER_DATA) {
            final var vd = (VillagerData) val;
            final var json = new JsonObject();
            final var type = Registries.VILLAGER_TYPE.getId(vd.getType()).getPath();
            final var profession = Registries.VILLAGER_PROFESSION.getId(vd.getProfession()).getPath();
            json.addProperty("type", type);
            json.addProperty("profession", profession);
            json.addProperty("level", vd.getLevel());
            return new Main.Pair<>("villager_data", json);
        } else if (handler == TrackedDataHandlerRegistry.OPTIONAL_INT) {
            final var opt = (OptionalInt) val;
            return new Main.Pair<>("optional_int", opt.isPresent() ? new JsonPrimitive(opt.getAsInt()) : JsonNull.INSTANCE);
        } else if (handler == TrackedDataHandlerRegistry.ENTITY_POSE) {
            return new Main.Pair<>("entity_pose", new JsonPrimitive(((EntityPose) val).name().toLowerCase(Locale.ROOT)));
        } else if (handler == TrackedDataHandlerRegistry.CAT_VARIANT) {
            return new Main.Pair<>("cat_variant",
                    new JsonPrimitive(((RegistryEntry<CatVariant>) val).getIdAsString()));
        } else if (handler == TrackedDataHandlerRegistry.WOLF_VARIANT) {
            return new Main.Pair<>("wolf_variant",
                    new JsonPrimitive(((RegistryEntry<WolfVariant>) val).getIdAsString()));
        } else if (handler == TrackedDataHandlerRegistry.FROG_VARIANT) {
            return new Main.Pair<>("frog_variant",
                    new JsonPrimitive(((RegistryEntry<FrogVariant>) val).getIdAsString()));
        } else if (handler == TrackedDataHandlerRegistry.OPTIONAL_GLOBAL_POS) {
            return new Main.Pair<>("optional_global_pos", ((Optional<?>) val).map(o -> {
                final var gp = (GlobalPos) o;
                final var json = new JsonObject();
                json.addProperty("dimension", gp.dimension().getValue().toString());

                final var posJson = new JsonObject();
                posJson.addProperty("x", gp.pos().getX());
                posJson.addProperty("y", gp.pos().getY());
                posJson.addProperty("z", gp.pos().getZ());

                json.add("position", posJson);
                return (JsonElement) json;
            }).orElse(JsonNull.INSTANCE));
        } else if (handler == TrackedDataHandlerRegistry.PAINTING_VARIANT) {
            final var variant = ((RegistryEntry<?>) val).getKey().map(k -> k.getValue().getPath()).orElse("");
            return new Main.Pair<>("painting_variant", new JsonPrimitive(variant));
        } else if (handler == TrackedDataHandlerRegistry.SNIFFER_STATE) {
            return new Main.Pair<>("sniffer_state",
                    new JsonPrimitive(((SnifferEntity.State) val).name().toLowerCase(Locale.ROOT)));
        } else if (handler == TrackedDataHandlerRegistry.ARMADILLO_STATE) {
            return new Main.Pair<>("armadillo_state",
                    new JsonPrimitive(((ArmadilloEntity.State) val).name().toLowerCase(Locale.ROOT)));
        } else if (handler == TrackedDataHandlerRegistry.VECTOR3F) {
            final var vec = (Vector3f) val;
            final var json = new JsonObject();
            json.addProperty("x", vec.x);
            json.addProperty("y", vec.y);
            json.addProperty("z", vec.z);
            return new Main.Pair<>("vector3f", json);
        } else if (handler == TrackedDataHandlerRegistry.QUATERNIONF) {
            final var quat = (Quaternionf) val;
            final var json = new JsonObject();
            json.addProperty("x", quat.x);
            json.addProperty("y", quat.y);
            json.addProperty("z", quat.z);
            json.addProperty("w", quat.w);
            return new Main.Pair<>("quaternionf", json);
        } else {
            throw new IllegalArgumentException(
                    "Unexpected tracked handler of ID " + TrackedDataHandlerRegistry.getId(handler) + handler.toString());
        }
    }

    @Override
    public String fileName() {
        return "entities.json";
    }

    @Override
    @SuppressWarnings("unchecked")
    public JsonElement extract() throws IllegalAccessException, NoSuchFieldException {

        var entityList = new ArrayList<Main.Pair<Class<? extends Entity>, EntityType<?>>>();
        final var entityClassTypeMap = new HashMap<Class<? extends Entity>, EntityType<?>>();
        for (final var f : EntityType.class.getFields()) {
            if (f.getType().equals(EntityType.class)) {
                final var entityClass = (Class<? extends Entity>) ((ParameterizedType) f.getGenericType())
                        .getActualTypeArguments()[0];
                final var entityType = (EntityType<?>) f.get(null);

                entityList.add(new Main.Pair<>(entityClass, entityType));
                entityClassTypeMap.put(entityClass, entityType);
            }
        }

        var dataTrackerField = Entity.class.getDeclaredField("dataTracker");
        dataTrackerField.setAccessible(true);

        final var entitiesMap = new TreeMap<Class<? extends Entity>, JsonElement>(new ClassComparator());

        for (final var entry : entityList) {
            var entityClass = entry.left();
            @Nullable
            var entityType = entry.right();
            assert null != entityType;

            // While we can use the tracked data registry and reflection to get the tracked
            // fields on entities, we won't know what their default values are because they
            // are assigned in the entity's constructor.
            // To obtain this, we create a dummy world to spawn the entities into and read
            // the data tracker field from the base entity class.
            // We also handle player entities specially since they cannot be spawned with
            // EntityType#create.
            var entityInstance = entityType.equals(EntityType.PLAYER) ? new DummyPlayerEntity(this.world, BlockPos.ofFloored(0, 70, 0), 0, new GameProfile(UUID.randomUUID(), "cooldude"), null)
                    : entityType.create(this.world);


            var dataTracker = (DataTracker) dataTrackerField.get(entityInstance);
//            final var dataTracker = entityInstance.getDataTracker();

            while (null == entitiesMap.get(entityClass)) {
                final var entityJson = new JsonObject();

                final var parent = entityClass.getSuperclass();
                final var hasParent = null != parent && Entity.class.isAssignableFrom(parent);

                if (hasParent) {
                    entityJson.addProperty("parent", parent.getSimpleName());
                }

                if (null != entityType) {
                    entityJson.addProperty("type", Registries.ENTITY_TYPE.getId(entityType).getPath());

                    entityJson.add("translation_key", new JsonPrimitive(entityType.getTranslationKey()));
                }

                final var fieldsJson = new JsonArray();
                for (final var entityField : entityClass.getDeclaredFields()) {
                    if (entityField.getType().equals(TrackedData.class)) {
                        entityField.setAccessible(true);

                        final var trackedData = (TrackedData<?>) entityField.get(null);


                        final var fieldJson = new JsonObject();
                        final var fieldName = entityField.getName().toLowerCase(Locale.ROOT);
                        fieldJson.addProperty("name", fieldName);
                        fieldJson.addProperty("index", trackedData.id());

                        final var data = trackedDataToJson(trackedData, dataTracker);
                        fieldJson.addProperty("type", data.left());
                        fieldJson.add("default_value", data.right());

                        fieldsJson.add(fieldJson);
                    }
                }
                entityJson.add("fields", fieldsJson);

                if (entityInstance instanceof final LivingEntity livingEntity) {
                    final var type = (EntityType<? extends LivingEntity>) entityType;
                    final var defaultAttributes = DefaultAttributeRegistry.get(type);
                    final var attributesJson = new JsonArray();
                    if (null != defaultAttributes) {
                        final var instancesField = defaultAttributes.getClass().getDeclaredField("instances");
                        instancesField.setAccessible(true);
                        final var instances = (Map<EntityAttribute, EntityAttributeInstance>) instancesField
                                .get(defaultAttributes);

                        for (final var instance : instances.values()) {
                            final var attribute = instance.getAttribute().value();

                            final var attributeJson = new JsonObject();

                            attributeJson.addProperty("id", Registries.ATTRIBUTE.getRawId(attribute));
                            attributeJson.addProperty("name", Registries.ATTRIBUTE.getId(attribute).getPath());
                            attributeJson.addProperty("base_value", instance.getBaseValue());

                            attributesJson.add(attributeJson);
                        }
                    }
                    entityJson.add("attributes", attributesJson);
                }

                final var bb = entityInstance.getBoundingBox();
                if (null != bb && null != entityType) {
                    final var boundingBoxJson = new JsonObject();

                    boundingBoxJson.addProperty("size_x", bb.getLengthX());
                    boundingBoxJson.addProperty("size_y", bb.getLengthY());
                    boundingBoxJson.addProperty("size_z", bb.getLengthZ());

                    entityJson.add("default_bounding_box", boundingBoxJson);
                }

                entitiesMap.put(entityClass, entityJson);

                if (!hasParent) {
                    break;
                }

                entityClass = (Class<? extends Entity>) parent;
                entityType = entityClassTypeMap.get(entityClass);
            }
        }

        final var entitiesJson = new JsonObject();
        for (final var entry : entitiesMap.entrySet()) {
            entitiesJson.add(entry.getKey().getSimpleName(), entry.getValue());
        }

        return entitiesJson;
    }
}
