#![allow(clippy::new_without_default)]

use anyways::{ext::AuditExt, Result};
use log::{info, LevelFilter};
use semver::Version;
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use ty::chunk_pos::ChunkPos;
use world::{
	chunk::{storage::ChunkStorage, Chunk},
	entity::EntityWorld,
};

use crate::{
	api::Api,
	debug::DummyRenderer,
	network::{packet::ServerBoundPacket, ServerNetwork},
	player::PlayerSystem,
	world::World,
};

pub mod api;
pub mod debug;
pub mod network;
pub mod player;
pub mod ty;
pub mod util;
pub mod world;
pub mod item;

pub const TPS: usize = 60;
pub const KERNEL_VERSION: Version = Version::new(0, 0, 1);

pub struct Server {
	network: ServerNetwork,
	player: PlayerSystem,
	world: World,
}

impl Server {
	pub fn new(api: &Api, network: ServerNetwork, world: World) -> Result<Server> {
		info!("Launching integrated server.");
		Ok(Server {
			network,
			player: PlayerSystem::new(api)?,
			world,
		})
	}

	pub fn tick(&mut self, api: &Api) -> Result<()> {
		for (token, packet) in self.network.poll() {
			match packet {
				ServerBoundPacket::Player(packet) => {
					self.player.packet(api, token, packet, &mut self.world);
				}
				ServerBoundPacket::World(packet) => {
					self.world.packet(api, token, packet, &mut self.network)?;
				}
			}
		}

		self.world
			.tick(api, &mut DummyRenderer)
			.wrap_err("Ticking world")?;
		self.player
			.tick(&mut self.network, &self.world)
			.wrap_err("Ticking player system.")?;
		Ok(())
	}
}

static mut INITILIZED: bool = false;
pub fn initialize() -> Result<()> {
	unsafe {
		if !INITILIZED {
			INITILIZED = true;
			TermLogger::init(
				LevelFilter::Trace,
				Config::default(),
				TerminalMode::Mixed,
				ColorChoice::Auto,
			)?;
			info!("Logging initialized successfully for Rustaria {}", KERNEL_VERSION.to_string());
		}
	}
	Ok(())
}