use std::mem;

use bevy_ecs::prelude::DetectChanges;
use bevy_ecs::query::QueryData;
use bevy_ecs::world::Ref;
use valence_math::DVec3;
use valence_protocol::encode::WritePacket;
use valence_protocol::movement_flags::MovementFlags;
use valence_protocol::packets::play::{
    AddEntityS2c, AnimateS2c, EntityEventS2c, MoveEntityPosRotS2c,
    MoveEntityPosS2c, MoveEntityRotS2c, RotateHeadS2c, SetEntityDataS2c, SetEntityMotionS2c,
    TeleportEntityS2c, UpdateAttributesS2c,
};
use valence_protocol::var_int::VarInt;
use valence_protocol::ByteAngle;
use valence_server_common::UniqueId;

use crate::attributes::TrackedEntityAttributes;
use crate::tracked_data::TrackedData;
use crate::{
    EntityAnimations, EntityId, EntityKind, EntityLayerId, EntityStatuses, HeadYaw, Look,
    ObjectData, OldEntityLayerId, OldPosition, OnGround, Position, Velocity,
};

#[derive(QueryData)]
pub struct EntityInitQuery {
    pub entity_id: &'static EntityId,
    pub uuid: &'static UniqueId,
    pub kind: &'static EntityKind,
    pub look: &'static Look,
    pub head_yaw: &'static HeadYaw,
    pub on_ground: &'static OnGround,
    pub object_data: &'static ObjectData,
    pub velocity: &'static Velocity,
    pub tracked_data: &'static TrackedData,
}

impl EntityInitQueryItem<'_> {
    /// Writes the appropriate packets to initialize an entity. This will spawn
    /// the entity and initialize tracked data. `pos` is the initial position of
    /// the entity.
    pub fn write_init_packets<W: WritePacket>(&self, pos: DVec3, mut writer: W) {
        match *self.kind {
            EntityKind::MARKER => {}
            _ => writer.write_packet(&AddEntityS2c {
                entity_id: self.entity_id.get().into(),
                object_uuid: self.uuid.0,
                kind: self.kind.get().into(),
                position: pos,
                pitch: ByteAngle::from_degrees(self.look.pitch),
                yaw: ByteAngle::from_degrees(self.look.yaw),
                head_yaw: ByteAngle::from_degrees(self.head_yaw.0),
                data: self.object_data.0.into(),
                velocity: self.velocity.to_packet_units(),
            }),
        }

        if let Some(init_data) = self.tracked_data.init_data() {
            writer.write_packet(&SetEntityDataS2c {
                entity_id: self.entity_id.get().into(),
                tracked_values: init_data.into(),
            });
        }
    }
}

#[derive(QueryData)]
pub struct UpdateEntityQuery {
    pub id: &'static EntityId,
    pub pos: &'static Position,
    pub old_pos: &'static OldPosition,
    pub loc: &'static EntityLayerId,
    pub old_loc: &'static OldEntityLayerId,
    pub look: Ref<'static, Look>,
    pub head_yaw: Ref<'static, HeadYaw>,
    pub on_ground: &'static OnGround,
    pub velocity: Ref<'static, Velocity>,
    pub tracked_data: &'static TrackedData,
    pub statuses: &'static EntityStatuses,
    pub animations: &'static EntityAnimations,
    // Option because not all entities have attributes, only LivingEntity.
    pub tracked_attributes: Option<&'static TrackedEntityAttributes>,
}

impl UpdateEntityQueryItem<'_> {
    pub fn write_update_packets<W: WritePacket>(&self, mut writer: W) {
        // TODO: @RJ I saw you're using UpdateEntityPosition and UpdateEntityRotation sometimes. These two packets are actually broken on the client and will erase previous position/rotation https://bugs.mojang.com/browse/MC-255263 -Moulberry

        let entity_id = VarInt(self.id.get());

        let position_delta = self.pos.0 - self.old_pos.get();
        let needs_teleport = position_delta.abs().max_element() >= 8.0;
        let changed_position = self.pos.0 != self.old_pos.get();

        if changed_position && !needs_teleport && self.look.is_changed() {
            writer.write_packet(&MoveEntityPosRotS2c {
                entity_id,
                delta: (position_delta * 4096.0).to_array().map(|v| v as i16),
                yaw: ByteAngle::from_degrees(self.look.yaw),
                pitch: ByteAngle::from_degrees(self.look.pitch),
                // FIXME: add missing pushing_against_wall
                flags: MovementFlags::new().with_on_ground(self.on_ground.0),
            });
        } else {
            if changed_position && !needs_teleport {
                writer.write_packet(&MoveEntityPosS2c {
                    entity_id,
                    delta: (position_delta * 4096.0).to_array().map(|v| v as i16),
                    on_ground: self.on_ground.0,
                });
            }

            if self.look.is_changed() {
                writer.write_packet(&MoveEntityRotS2c {
                    entity_id,
                    yaw: ByteAngle::from_degrees(self.look.yaw),
                    pitch: ByteAngle::from_degrees(self.look.pitch),
                    on_ground: self.on_ground.0,
                });
            }
        }

        if needs_teleport {
            writer.write_packet(&TeleportEntityS2c {
                entity_id,
                position: self.pos.0,
                yaw: ByteAngle::from_degrees(self.look.yaw),
                pitch: ByteAngle::from_degrees(self.look.pitch),
                on_ground: self.on_ground.0,
            });
        }

        if self.velocity.is_changed() {
            writer.write_packet(&SetEntityMotionS2c {
                entity_id,
                velocity: self.velocity.to_packet_units(),
            });
        }

        if self.head_yaw.is_changed() {
            writer.write_packet(&RotateHeadS2c {
                entity_id,
                head_yaw: ByteAngle::from_degrees(self.head_yaw.0),
            });
        }

        if let Some(update_data) = self.tracked_data.update_data() {
            writer.write_packet(&SetEntityDataS2c {
                entity_id,
                tracked_values: update_data.into(),
            });
        }

        if self.statuses.0 != 0 {
            for i in 0..mem::size_of_val(self.statuses) {
                if (self.statuses.0 >> i) & 1 == 1 {
                    writer.write_packet(&EntityEventS2c {
                        entity_id: entity_id.0,
                        entity_status: i as u8,
                    });
                }
            }
        }

        if self.animations.0 != 0 {
            for i in 0..mem::size_of_val(self.animations) {
                if (self.animations.0 >> i) & 1 == 1 {
                    writer.write_packet(&AnimateS2c {
                        entity_id,
                        animation: i as u8,
                    });
                }
            }
        }

        if let Some(attributes) = self.tracked_attributes {
            let properties = attributes.get_properties();

            if !properties.is_empty() {
                writer.write_packet(&UpdateAttributesS2c {
                    entity_id,
                    properties,
                });
            }
        }
    }
}
