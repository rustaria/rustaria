use std::collections::HashMap;

use anyways::{ext::AuditExt, Result};
use apollo::{macros::*, FromLua};

use crate::{
	api::prototype::Prototype,
	ty::{id::Id, identifier::Identifier},
	world::chunk::spread::{BlockSpreader, BlockSpreaderPrototype},
};

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Block {
	pub id: Id<BlockDesc>,
	pub collision: bool,
}

#[lua_impl]
impl Block {
	#[lua_method]
	pub fn get_id(&self) -> Id<BlockDesc> { self.id }

	#[lua_method]
	pub fn get_collision(&self) -> bool { self.collision }
}

#[derive(Clone)]
pub struct BlockDesc {
	pub collision: bool,
	pub spread: Option<BlockSpreader>,
}

#[lua_impl]
impl BlockDesc {
	#[lua_method]
	pub fn create(&self, id: Id<BlockDesc>) -> Block {
		Block {
			id,
			collision: self.collision,
		}
	}
}

#[derive(FromLua, Debug)]
pub struct BlockPrototype {
	pub collision: bool,
	pub spread: Option<BlockSpreaderPrototype>,
}

#[lua_impl]
impl BlockPrototype {
	pub fn bake(self, blocks: &HashMap<Identifier, Id<BlockDesc>>) -> Result<BlockDesc> {
		Ok(BlockDesc {
			collision: self.collision,
			spread: if let Some(spread) = self.spread {
				Some(spread.bake(blocks).wrap_err("Could not bake spreader")?)
			} else {
				None
			},
		})
	}
}

impl Prototype for BlockPrototype {
	type Output = BlockDesc;

	fn get_name() -> &'static str { "block" }
}