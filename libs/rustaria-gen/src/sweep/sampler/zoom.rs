use std::ops::Range;
use crate::Sweep;
use crate::sweep::sampler::Sampler;

#[derive(Clone)]
/// Extends the inner samplers range.
/// - anything below `start` will be 0,
/// - anything above `end` will be 1,
/// - anything between `start`..`end` will be scaled to those values. so a inner of 0.5 will give the value that is between `start` and `end`
pub struct ZoomSampler {
	pub(crate) range: Range<f32>,
	pub(crate) sampler: Sampler,
}

impl ZoomSampler {
	pub fn new(range: Range<f32>, sampler: Sampler) -> Box<ZoomSampler> {
		Box::new(ZoomSampler { range, sampler })
	}

	pub fn get<T: Clone + Default>(&self, sweep: &Sweep<T>, x: u32, y: u32) -> f32 {
		let low = (self.sampler.get(sweep, x, y) - self.range.start).max(0.0);
		(low / (self.range.end - self.range.start)).clamp(0.0, 1.0)
	}
}

