use std::collections::HashMap;

use glium::{uniform, Blend, DrawParameters, Program};
use layer::BlockLayerRenderer;
use rsa_client_core::{
	debug::Debug,
	frontend::Frontend,
	ty::{Draw, MeshBuilder, MeshDrawer, PosTexVertex},
};
use rsa_core::{
	debug::DebugCategory,
	draw_debug,
	err::Result,
	math::rect,
	ty::{DirMap, Direction, Offset},
};
use rsa_registry::Storage;
use rsa_world::{
	chunk::{layer::BlockLayer, storage::ChunkStorage},
	ty::{BlockPos, ChunkPos},
	CHUNK_SIZE,
};

use crate::GraphicsRPC;

pub mod block;
pub mod layer;

pub struct WorldChunkRenderer {
	chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
}

impl WorldChunkRenderer {
	pub fn new() -> Result<WorldChunkRenderer> {
		Ok(WorldChunkRenderer {
			chunk_meshes: Default::default(),
		})
	}

	pub fn reload(&mut self) { self.chunk_meshes.clear() }

	pub fn tick(&mut self, frontend: &Frontend, chunks: &ChunkStorage) -> Result<()> {
		for pos in chunks.get_dirty() {
			if let Some(renderer) = self.chunk_meshes.get_mut(pos) {
				renderer.dirty = true;
			} else {
				self.chunk_meshes.insert(
					*pos,
					ChunkMesh {
						drawer: frontend.create_drawer()?,
						builder: MeshBuilder::new(),
						dirty: true,
					},
				);
			}
		}
		Ok(())
	}

	pub fn draw(
		&mut self,
		rpd: &GraphicsRPC,
		chunk: &ChunkStorage,
		program: &Program,
		draw: &mut Draw,
	) -> Result<()> {
		let uniforms = uniform! {
			screen_ratio: draw.frontend.aspect_ratio,
			atlas: &draw.atlas.texture,
			player_pos: draw.viewport.pos.to_array(),
			zoom: draw.viewport.zoom,
		};
		let draw_parameters = DrawParameters {
			blend: Blend::alpha_blending(),
			..DrawParameters::default()
		};

		{
			let cs = CHUNK_SIZE as f32;
			let view = draw.viewport.rect.scale(1.0 / cs, 1.0 / cs).round_out();
			let y_min = view.origin.y as i64;
			let y_max = view.origin.y as i64 + view.size.height as i64;
			let x_min = view.origin.x as i64;
			let x_max = view.origin.x as i64 + view.size.width as i64;
			for y in y_min..y_max {
				for x in x_min..x_max {
					if let Ok(pos) = ChunkPos::try_from((x, y)) {
						draw_debug!(
							draw.debug,
							DebugCategory::ChunkBorders,
							rect(x as f32 * 16.0, y as f32 * 16.0, 16.0, 16.0)
						);

						if let Some(render) = self.chunk_meshes.get_mut(&pos) {
							render.tick(pos, chunk, &rpd.block_layer_renderer, draw.debug)?;
							render
								.drawer
								.draw(draw.frame, program, &uniforms, &draw_parameters)?;
						} else {
							draw_debug!(
								draw.debug,
								DebugCategory::Temporary,
								rect(x as f32 * 16.0, y as f32 * 16.0, 16.0, 16.0),
								0xff0000,
								1.0,
								0.5
							);
							self.chunk_meshes.insert(
								pos,
								ChunkMesh {
									drawer: draw.frontend.create_drawer()?,
									builder: MeshBuilder::new(),
									dirty: true,
								},
							);
						}
					}
				}
			}
		}

		Ok(())
	}
}

pub struct ChunkMesh {
	drawer: MeshDrawer<PosTexVertex>,
	builder: MeshBuilder<PosTexVertex>,
	dirty: bool,
}

impl ChunkMesh {
	pub fn tick(
		&mut self,
		pos: ChunkPos,
		chunks: &ChunkStorage,
		renderers: &Storage<Option<BlockLayerRenderer>, BlockLayer>,
		debug: &mut Debug,
	) -> Result<()> {
		if self.dirty {
			draw_debug!(
				debug,
				DebugCategory::ChunkMeshing,
				rect(
					pos.x as f32 * CHUNK_SIZE as f32,
					pos.y as f32 * CHUNK_SIZE as f32,
					CHUNK_SIZE as f32,
					CHUNK_SIZE as f32,
				),
				0xffffff,
				2.0,
				0.5
			);
			if let Some(chunk) = chunks.get(pos) {
				let mut neighbors = DirMap::new([None; 4]);
				for dir in Direction::values() {
					if let Some(pos) = pos.checked_offset(dir) {
						if let Some(chunk) = chunks.get(pos) {
							neighbors[dir] = Some(chunk);
						}
					}
				}

				for (id, layer) in chunk.layers.iter() {
					if let Some(renderer) = &renderers[id]{
						renderer.mesh_chunk_layer(
							pos,
							layer,
							neighbors.map(|_, option| option.map(|c| &c.layers[id])),
							&mut self.builder,
							debug,
						);
					}
				}
			}
			self.drawer.upload(&self.builder)?;
			self.builder.clear();
			self.dirty = false;
		}
		Ok(())
	}
}

fn get_variation(pos: BlockPos) -> u32 {
	let x = (pos.x() & 0xFFFFFFFF) as u32;
	let y = (pos.y() & 0xFFFFFFFF) as u32;
	let offset_x = x.overflowing_mul(69).0;
	let mut v = offset_x.overflowing_mul(y + 420).0;
	v ^= v.overflowing_shl(13).0;
	v ^= v.overflowing_shr(7).0;
	v ^= v.overflowing_shl(17).0;
	v
}
