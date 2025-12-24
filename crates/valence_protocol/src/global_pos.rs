use valence_ident::Ident;
use valence_protocol_macros::HashOps;
use crate::block_pos::BlockPos;
use crate::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, HashOps)]
pub struct GlobalPos {
    pub dimension_name: Ident<String>,
    pub position: BlockPos,
}
