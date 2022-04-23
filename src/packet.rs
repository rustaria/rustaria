use crate::packet::chunk::{ClientChunkPacket, ServerChunkPacket};
use crate::packet::entity::{ClientEntityPacket, ServerEntityPacket};
use rustaria_network::{EstablishingInstance, EstablishingStatus, Packet, Result};
use serde::{Deserialize, Serialize};

pub mod chunk;
pub mod entity;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerPacket {
	Chunk(ServerChunkPacket),
	Entity(ServerEntityPacket),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientPacket {
	Chunk(ClientChunkPacket),
	Entity(ClientEntityPacket),
}

impl Packet for ServerPacket {}
impl Packet for ClientPacket {}

pub struct PlayerJoinInstance {}

impl EstablishingInstance<PlayerJoinData> for PlayerJoinInstance {
	fn receive(&mut self, _data: &[u8]) -> Result<EstablishingStatus<PlayerJoinData>> {
		todo!()
	}
}

pub struct PlayerJoinData {}