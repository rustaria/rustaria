use std::collections::HashSet;

use rustaria_api::ty::Tag;
use rustaria_api::ty::{Prototype, RawId};
use rustaria_util::ty::pos::Pos;

#[cfg(feature = "client")]
use crate::api::rendering::RenderingSystem;
use crate::entity::VelocityComp;

#[derive(Clone, Debug)]
pub struct EntityPrototype {
	pub velocity: Option<VelocityCompPrototype>,
	#[cfg(feature = "client")]
	pub rendering: Option<RenderingSystem>,
}

impl Prototype for EntityPrototype {
	type Item = ();

	fn create(&self, _: RawId) -> Self::Item {}

	fn get_sprites(&self, sprites: &mut HashSet<Tag>) {
		#[cfg(feature = "client")]
		if let Some(system) = &self.rendering {
			match system {
				RenderingSystem::Static(pane) => {
					sprites.insert(pane.sprite.clone());
				}
				RenderingSystem::State(states) => {
					for pane in states.values() {
						sprites.insert(pane.sprite.clone());
					}
				}
			}
		}
	}

	fn lua_registry_name() -> &'static str {
		"Entities"
	}
}

#[derive(Clone, Debug, Default)]
pub struct VelocityCompPrototype {
	pub x: f32,
	pub y: f32,
}

impl Prototype for VelocityCompPrototype {
	type Item = VelocityComp;

	fn create(&self, _: RawId) -> Self::Item {
		VelocityComp {
			velocity: Pos {
				x: self.x,
				y: self.y,
			},
		}
	}
}
