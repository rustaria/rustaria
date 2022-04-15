use std::collections::HashSet;
use std::sync::Arc;

use crossbeam::channel::{Receiver, Sender, unbounded};
use rayon::{ThreadPool};

use rustaria_api::Carrier;
use rustaria_util::{error, Result};
use rustaria_util::ty::ChunkPos;

use crate::world::chunk::Chunk;
use crate::world::gen::chunk::generate_chunk;

mod chunk;

pub struct ChunkGenerator {
	carrier: Carrier,
	thread_pool: Arc<ThreadPool>,
	submitted_chunks: HashSet<ChunkPos>,
	channel: (Sender<(Chunk, ChunkPos)>, Receiver<(Chunk, ChunkPos)>),
}

impl ChunkGenerator {
	pub fn new(carrier: Carrier, thread_pool: Arc<ThreadPool>) -> Result<ChunkGenerator> {
		Ok(ChunkGenerator {
			carrier,
			thread_pool,
			submitted_chunks: Default::default(),
			channel: unbounded()
		})
	}

	pub fn request_chunk(&mut self, pos: ChunkPos) {
		if !self.submitted_chunks.contains(&pos) {
			self.submitted_chunks.insert(pos);
			let carrier = self.carrier.clone();
			let sender = self.channel.0.clone() ;
			self.thread_pool.spawn(move || {
				let api = carrier;
				match generate_chunk(&api, pos) {
					Ok(chunk) => sender.send((chunk, pos)).unwrap(),
					Err(err) => {
						error!("Could not generate chunk {}", err);
					}
				};
			});
		}
	}

	pub fn poll_chunks<C: FnMut(Chunk, ChunkPos)>(&mut self, mut func: C) {
		while let Ok((chunk, pos)) = self.channel.1.try_recv() {
			self.submitted_chunks.remove(&pos);
			func(chunk, pos);
		}
	}
}


