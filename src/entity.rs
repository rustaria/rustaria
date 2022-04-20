pub mod hitbox;
pub mod pos;
pub mod velocity;

use std::collections::{HashMap, HashSet};

use eyre::{ContextCompat, Result};
use pos::PositionComp;

use rustaria_api::ty::{Prototype, RawId};
use rustaria_api::{Carrier, Reloadable};
use rustaria_util::ty::pos::Pos;
use rustaria_util::{error, info, Uuid};
use velocity::VelocityComp;

use crate::api::prototype::entity::EntityPrototype;
use crate::chunk::ChunkWorld;
use crate::entity::hitbox::HitboxComp;
use crate::{ChunkManager, SmartError};

#[derive(Default)]
pub struct EntityWorld {
	carrier: Option<Carrier>,
	pub entities: HashMap<Uuid, RawId>,
	pub position: HashMap<Uuid, PositionComp>,
	pub velocity: HashMap<Uuid, VelocityComp>,
	pub hitbox: HashMap<Uuid, HitboxComp>,
	pub dead: HashSet<Uuid>,
}

impl EntityWorld {
	pub fn spawn(&mut self, id: RawId, pos: Pos) -> Result<Uuid> {
		info!("spawn");
		let carrier = self
			.carrier
			.as_ref()
			.wrap_err(SmartError::CarrierUnavailable)?
			.lock();

		// Get uuid and handle conflicts by re-rolling until you find a spot.
		let mut uuid = Uuid::new_v4();
		while self.entities.contains_key(&uuid) {
			uuid = Uuid::new_v4();
		}
		self.entities.insert(uuid, id);

		// Get prototype
		let prototype = carrier
			.get_registry::<EntityPrototype>()
			.prototype_from_id(id)
			.wrap_err("Could not find entity")?;

		// Add components
		self.position.insert(uuid, PositionComp { position: pos });

		if let Some(hitbox) = &prototype.hitbox {
			self.hitbox.insert(uuid, hitbox.clone());
		}

		if let Some(velocity) = &prototype.velocity {
			self.velocity.insert(uuid, velocity.clone());
		}

		Ok(uuid)
	}

	pub fn kill(&mut self, id: Uuid) {
		self.entities.remove(&id);
		self.position.remove(&id);
		self.velocity.remove(&id);
		self.hitbox.remove(&id);
	}

	pub fn tick(&mut self, chunks: &ChunkWorld) {
		for id in self.dead.drain() {
			self.entities.remove(&id);
			self.position.remove(&id);
			self.velocity.remove(&id);
			self.hitbox.remove(&id);
		}

		for (id, velocity) in &mut self.velocity {
			// required
			if let Some(position) = self.position.get_mut(id) {
				// optional
				if let Some(hitbox) = self.hitbox.get(id) {
					if let Some((pos, hit)) =
						hitbox.calc(hitbox.hitbox, position.position, velocity.velocity, chunks)
					{
						position.position = pos;
						if hit {
							self.dead.insert(*id);
						}
					}
				} else {
					position.position += velocity.velocity;
				}
			}
		}
	}
}

impl Reloadable for EntityWorld {
	fn reload(&mut self, _: &rustaria_api::Api, carrier: &Carrier) {
		self.carrier = Some(carrier.clone());
	}
}
