use std::sync::{Arc, Mutex};

use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

/// Keeps track of the most recently played samples in a ring buffer for visualisation.
///
/// This is not a good solution for getting data from the audio thread to the UI thread.
#[derive(Clone, Default)]
pub struct SampleTracker {
    samples: Arc<Mutex<ConstGenericRingBuffer<f64, 4410>>>,
}

impl SampleTracker {
    pub fn add_sample(&self, sample: f64) {
        let mut samples = self.samples.lock().unwrap();
        samples.enqueue(sample);
    }

    pub fn samples_vec(&self) -> Vec<f64> {
        let samples = self.samples.lock().unwrap();
        samples.to_vec()
    }
}
