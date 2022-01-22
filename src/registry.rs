#![allow(unused)] // alpha, remove this when you're done - leocth

use std::collections::HashMap;

use crate::api::plugin::ArchivePath;
use crate::chunk::tile::TilePrototype;
use crate::chunk::wall::WallPrototype;
use bimap::BiHashMap;
use eyre::{Report, Result};
use serde::Deserialize;
use tracing::debug;

pub struct Registry<P> {
    name: &'static str,
    tag_to_id: BiHashMap<Tag, Id>,
    entries: Vec<P>,
    current_id: u32,
}

impl<P> Registry<P> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            tag_to_id: Default::default(),
            entries: Default::default(),
            current_id: 0,
        }
    }

    pub fn register(&mut self, tag: Tag, prototype: P) -> Id {
        let name = self.name;
        debug!(target: "registry", "{}: Registered {:?}", name, tag);
        let id = Id(self.current_id);
        self.tag_to_id.insert(tag, id);
        self.entries.insert(self.current_id as usize, prototype);
        self.current_id += 1;
        id
    }

    pub fn entries(&self) -> impl Iterator<Item = &P> {
        self.entries.iter()
    }

    pub fn get_id(&self, tag: &Tag) -> Option<&Id> {
        self.tag_to_id.get_by_left(tag)
    }

    pub fn get_tag(&self, id: &Id) -> Option<&Tag> {
        self.tag_to_id.get_by_right(id)
    }

    pub fn get_entry(&self, id: &Id) -> Option<&P> {
        self.entries.get(id.0 as usize)
    }
}

// This is lua input (or rust) that gets converted to id,
// by the registry map.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct Tag(String, String);

impl Tag {
    pub fn new(mod_id: String, string: String) -> Self {
        Self(mod_id, string)
    }

    pub fn parse(string: &str) -> eyre::Result<Self> {
        if let Some(colon) = string.find(':') {
            let (mod_id, obj_id) = string.split_at(colon);
            Ok(Self(mod_id.to_string(), obj_id[1..].to_string()))
        } else {
            Err(Report::msg("Could not find delimiter :"))
        }
    }
}

// kernel identification
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Id(pub u32);

#[derive(Clone, Debug, Deserialize)]
pub struct LanguageKey {
    // TODO
}

#[derive(Clone, Debug, Deserialize)]
pub struct AssetLocation(pub String, pub String);