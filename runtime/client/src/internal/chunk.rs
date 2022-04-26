use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use rustaria::chunk::ChunkStorage;
use rustaria::ClientNetwork;
use rustaria::packet::chunk::{ClientChunkPacket, ServerChunkPacket};
use rustaria::packet::ClientPacket;
use rustaria_api::{Api, Carrier, Reloadable};
use rustaria_util::error::Result;
use rustaria_util::logging::warn;
use rustaria_util::math::vec2;
use rustaria_util::ty::{CHUNK_SIZE_F, ChunkPos, Offset};
use rustariac_backend::ty::Camera;
use rustariac_rendering::chunk_drawer::WorldChunkDrawer;

use crate::RenderingHandler;

pub(crate) struct ChunkHandler {
	rendering: RenderingHandler,

	storage: ChunkStorage,
	stored_chunks: HashSet<ChunkPos>,
	drawer: WorldChunkDrawer,

	old_chunk: ChunkPos,
	old_zoom: f32,
}

impl ChunkHandler {
	pub fn new(rendering: &RenderingHandler) -> ChunkHandler {
		ChunkHandler {
			rendering: rendering.clone(),
			storage: Default::default(),
			stored_chunks: Default::default(),
			drawer: WorldChunkDrawer::new(&rendering.backend),
			old_chunk: ChunkPos { x: 60, y: 420 },
			old_zoom: 0.0,
		}
	}

	pub fn packet(&mut self, packet: ServerChunkPacket) -> Result<()> {
		match packet {
			ServerChunkPacket::Provide(chunks) => match chunks.export() {
				Ok(chunks) => {
					for (pos, chunk) in chunks.chunks {
						self.drawer.submit(pos, &chunk)?;
						self.storage.put_chunk(pos, chunk);
					}
				}
				Err(chunks) => {
					warn!(target: "misc@rustariac", "Could not deserialize chunks packet. {chunks}")
				}
			},
		}

		Ok(())
	}

	pub fn tick(&mut self, camera: &Camera, networking: &mut ClientNetwork) -> Result<()> {
		if let Ok(chunk) = ChunkPos::try_from(vec2::<_, ()>(camera.position[0], camera.position[1]))
		{
			if chunk != self.old_chunk || camera.zoom != self.old_zoom || self.drawer.dirty() {
				let width = (camera.zoom / CHUNK_SIZE_F) as i32;
				let height = ((camera.zoom * camera.screen_y_ratio) / CHUNK_SIZE_F) as i32;
				let mut requested = Vec::new();
				for x in -width..width {
					for y in -height..height {
						if let Some(pos) = chunk.checked_offset((x, y)) {
							if !self.stored_chunks.contains(&pos) {
								requested.push(pos);
								self.stored_chunks.insert(pos);
							}
						}
					}
				}

				self.drawer.mark_dirty();
				if !requested.is_empty() {
					networking.send(ClientPacket::Chunk(ClientChunkPacket::Request(requested)))?;
				}
				self.old_chunk = chunk;
				self.old_zoom = camera.zoom;
			}
		}

		Ok(())
	}

	pub fn draw(&mut self, camera: &Camera) {
		self.drawer.draw(camera);
	}
}

impl Reloadable for ChunkHandler {
	fn reload(&mut self, api: &Api, carrier: &Carrier) {
		self.storage.clear();
		self.drawer.reload(api, carrier);
	}
}

impl Deref for ChunkHandler {
	type Target = ChunkStorage;

	fn deref(&self) -> &Self::Target {
		&self.storage
	}
}

impl DerefMut for ChunkHandler {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.storage
	}
}
