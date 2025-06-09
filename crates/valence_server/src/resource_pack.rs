use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use uuid::Uuid;
use valence_protocol::packets::play::{ResourcePackC2s, ResourcePackPushS2c};
use valence_protocol::text::Text;
use valence_protocol::WritePacket;

use crate::client::Client;
use crate::event_loop::{EventLoopPreUpdate, PacketEvent};

pub struct ResourcePackPlugin;

impl Plugin for ResourcePackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResourcePackStatusEvent>()
            .add_systems(EventLoopPreUpdate, handle_resource_pack_status);
    }
}

#[derive(Event, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ResourcePackStatusEvent {
    pub client: Entity,
    pub status: ResourcePackC2s,
}

impl Client {
    /// Requests that the client download and enable a resource pack.
    ///
    /// # Arguments
    /// * `url` - The URL of the resource pack file.
    /// * `hash` - The SHA-1 hash of the resource pack file. The value must be a
    ///   40-character hexadecimal string.
    /// * `forced` - Whether a client should be kicked from the server upon
    ///   declining the pack (this is enforced client-side)
    /// * `prompt_message` - A message to be displayed with the resource pack
    ///   dialog.
    /// # Returns
    /// - a [UUID](Uuid) that can be used to identify the resource pack.
    pub fn set_resource_pack(
        &mut self,
        url: &str,
        hash: &str,
        forced: bool,
        prompt_message: Option<Text>,
    ) -> Uuid {
        let uuid = Uuid::new_v4();

        self.write_packet(&ResourcePackPushS2c {
            uuid,
            url: url.into(),
            hash: hash.into(),
            forced,
            prompt_message: prompt_message.map(|t| t.into()),
        });

        uuid
    }

    /// Requests that the client download and enable a resource pack.
    ///
    /// # Arguments
    /// * `uuid` - The UUID to identify the resource pack.
    /// * `url` - The URL of the resource pack file.
    /// * `hash` - The SHA-1 hash of the resource pack file. The value must be a
    ///  40-character hexadecimal string.
    /// * `forced` - Whether a client should be kicked from the server upon
    /// declining the pack (this is enforced client-side)
    /// * `prompt_message` - A message to be displayed with the resource pack
    /// dialog.
    pub fn set_resource_pack_with_uuid(
        &mut self,
        uuid: Uuid,
        url: &str,
        hash: &str,
        forced: bool,
        prompt_message: Option<Text>,
    ) {
        self.write_packet(&ResourcePackPushS2c {
            uuid,
            url: url.into(),
            hash: hash.into(),
            forced,
            prompt_message: prompt_message.map(|t| t.into()),
        });
    }
}

fn handle_resource_pack_status(
    mut packets: EventReader<PacketEvent>,
    mut events: EventWriter<ResourcePackStatusEvent>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<ResourcePackC2s>() {
            events.send(ResourcePackStatusEvent {
                client: packet.client,
                status: pkt,
            });
        }
    }
}
