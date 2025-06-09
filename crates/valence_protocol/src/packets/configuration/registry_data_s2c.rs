use std::borrow::Cow;
use std::collections::HashMap;

use valence_ident::Ident;
use valence_nbt::Compound;

use crate::{Decode, Encode, Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
// After the server and the client have negotiated the required registry data,
// the server sends this packet for each registry to the client.
pub struct RegistryDataS2c<'a> {
    // The id of the registry
    pub id: Ident<Cow<'a, str>>,
    // The id of the entries and the entry data itself
    pub entries: HashMap<Ident<Cow<'a, str>>, Option<Compound>>,
}
