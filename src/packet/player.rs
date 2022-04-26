use serde::{Deserialize, Serialize};

use rustaria_util::math::{Vector2D, WorldSpace};
use rustaria_util::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerPlayerPacket {
	Attach {
		entity: Uuid,
		pos: Vector2D<f32, WorldSpace>,
	},
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientPlayerPacket {
	/// Creates a Player Entity
	Join {
		// inventory and stuff
	},
	SetPos(Vector2D<f32, WorldSpace>),
}
