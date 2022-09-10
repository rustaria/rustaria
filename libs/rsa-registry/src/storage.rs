use std::{
	marker::PhantomData,
	ops::{Index, IndexMut},
	vec::IntoIter,
};
use std::slice::{Iter, IterMut};

use crate::id::Id;

/// A Storage holds types mapped to an Id.
#[derive(Debug, Clone)]
pub struct Storage<V, I = V> {
	pub(crate) values: Vec<V>,
	pub(crate) _k: PhantomData<(I)>,
}

impl<V, I> Storage<V, I> {
	pub fn iter(&self) -> StorageIter<I, &'_ V, Iter<'_, V>> {
		StorageIter::new(self.values.iter())
	}

	pub fn iter_mut(&mut self) -> StorageIter<I, &'_ mut V, IterMut<'_, V>> {
		StorageIter::new(self.values.iter_mut())
	}
}

impl<V, I> Index<Id<I>> for Storage<V, I> {
	type Output = V;

	fn index(&self, index: Id<I>) -> &Self::Output {
		&self.values[index.key as usize]
	}
}

impl<V, I> IndexMut<Id<I>> for Storage<V, I> {
	fn index_mut(&mut self, index: Id<I>) -> &mut Self::Output {
		&mut self.values[index.key as usize]
	}
}

impl<V, I> Default for Storage<V, I> {
	fn default() -> Self {
		Storage {
			values: Vec::new(),
			_k: Default::default(),
		}
	}
}

impl<V, I> FromIterator<(Id<I>, V)> for Storage<V, I> {
	// this breaks the 100% safety but makes things 100x easier to use sooo ehm. yes
	fn from_iter<T: IntoIterator<Item = (Id<I>, V)>>(iter: T) -> Self {
		let mut items = Vec::new();
		for (id, value) in iter {
			items.push((id, value));
		}

		items.sort_by(|(id0, _), (id1, _)| id0.index().cmp(&id1.index()));

		// safety check
		let mut last_id: Option<usize> = None;
		for (id, _) in &items {
			if let Some(last_id) = last_id {
				if id.index() == last_id {
					panic!("Duplicate id");
				} else if id.index() != last_id + 1 {
					panic!("Skipped id");
				}
			} else if id.index() != 0 {
				panic!("id does not start on 0");
			}

			last_id = Some(id.index());
		}

		Storage {
			values: items.into_iter().map(|(_, value)| value).collect(),
			_k: Default::default(),
		}
	}
}

impl<V, I> IntoIterator for Storage<V, I> {
	type Item = (Id<I>, V);
	type IntoIter = StorageIter<I, V, IntoIter<V>>;

	fn into_iter(self) -> Self::IntoIter {
		StorageIter::new(self.values.into_iter())
	}
}

pub struct StorageIter<I, V, Iter: Iterator<Item = V>> {
	iter: Iter,
	id: u32,
	_p: PhantomData<(I)>,
}

impl<I, V, Iter: Iterator<Item = V>> StorageIter<I, V, Iter> {
	pub(crate) fn new(iter: Iter) -> StorageIter<I, V, Iter> {
		StorageIter {
			iter,
			id: 0,
			_p: Default::default(),
		}
	}
}

impl<I, V, Iter: Iterator<Item = V>> Iterator for StorageIter<I, V, Iter> {
	type Item = (Id<I>, V);

	fn next(&mut self) -> Option<Self::Item> {
		let out = self.iter.next().map(|v| {
			(
				Id::new(self.id),
				v,
			)
		});
		self.id += 1;
		out
	}
}

use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<V: Serialize, I> Serialize for Storage<V, I> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.values.serialize(serializer)
	}
}

impl<'de, V: Deserialize<'de>, I> Deserialize<'de> for Storage<V, I> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Vec::<V>::deserialize(deserializer).map(|lookup| Storage {
			values: lookup,
			_k: Default::default(),
		})
	}
}
