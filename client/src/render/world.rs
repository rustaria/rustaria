use anyways::Result;
use chunk::WorldChunkRenderer;
use glium::{program::SourceCode, Frame, Program};
use rustaria::world::World;

use crate::{
	render::{
		ty::{draw::Draw, viewport::Viewport},
		world::entity::WorldEntityRenderer,
	},
	ClientApi, Debug, Frontend, PlayerSystem, Timing,
};

pub mod chunk;
pub mod entity;
pub mod neighbor;

pub struct WorldRenderer {
	pos_color_program: Program,
	chunk_renderer: WorldChunkRenderer,
	entity_renderer: WorldEntityRenderer,
}

impl WorldRenderer {
	pub fn reload(&mut self) { self.chunk_renderer.reload(); }
}

impl WorldRenderer {
	pub fn new(frontend: &Frontend, _api: &ClientApi) -> Result<Self> {
		Ok(Self {
			pos_color_program: Program::new(
				&frontend.ctx,
				SourceCode {
					vertex_shader: include_str!("../builtin/pos_tex.vert.glsl"),
					tessellation_control_shader: None,
					tessellation_evaluation_shader: None,
					geometry_shader: None,
					fragment_shader: include_str!("../builtin/pos_tex.frag.glsl"),
				},
			)?,
			chunk_renderer: WorldChunkRenderer::new()?,
			entity_renderer: WorldEntityRenderer::new(frontend)?,
		})
	}

	pub fn tick(
		&mut self,
		frontend: &Frontend,
		_player: &PlayerSystem,
		world: &World,
		_debug: &mut Debug,
	) -> Result<()> {
		self.chunk_renderer.tick(frontend, &world.chunks)?;
		Ok(())
	}

	pub fn draw(
		&mut self,
		api: &ClientApi,
		frontend: &Frontend,
		player: &PlayerSystem,
		world: &World,
		frame: &mut Frame,
		viewport: &Viewport,
		debug: &mut Debug,
		timing: &Timing,
	) -> Result<()> {
		let mut draw = Draw {
			frame,
			viewport,
			atlas: api.atlas.as_ref().unwrap(),
			frontend,
			debug,
			timing,
		};
		self.chunk_renderer
			.draw(api, &world.chunks, &self.pos_color_program, &mut draw)?;
		self.entity_renderer.draw(
			api,
			player,
			&world.entities,
			&self.pos_color_program,
			&mut draw,
		)?;
		Ok(())
	}
}
