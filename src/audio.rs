use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, Device, SampleFormat, SizedSample, Stream, StreamConfig
};
use dasp_sample::Duplex;
use fundsp::hacker::{AudioUnit, adsr_live, sine, var};

use crate::{sample_tracker::SampleTracker, SharedControls};

/// Defines that actual synth that is controlled by the shared MIDI controls.
pub fn create_sound(
    SharedControls {
        pitch,
        volume,
        pitch_bend,
        control,
    }: SharedControls,
) -> Box<dyn AudioUnit> {
    Box::new(
        var(&pitch_bend) * var(&pitch)
            >> sine() * (var(&control) >> adsr_live(0.1, 0.2, 0.4, 0.2)) * var(&volume),
    )
}

/// Finds the default output device and runs the synth in it's expected sample format.
pub fn run_on_default_device(
    controls: SharedControls,
    sample_tracker: SampleTracker,
) -> anyhow::Result<Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or(anyhow::anyhow!("No output audio device found"))?;
    let config = device.default_output_config()?;

    match config.sample_format() {
        SampleFormat::F32 => run_synth::<f32>(device, config.into(), controls, sample_tracker),
        SampleFormat::I16 => run_synth::<i16>(device, config.into(), controls, sample_tracker),
        SampleFormat::U16 => run_synth::<u16>(device, config.into(), controls, sample_tracker),
        _ => panic!("Unsupported format"),
    }
}

/// Streams the audio to the given audio device.
/// This should be spawned on it's own thread and the thread should be kept alive.
fn run_synth<T: SizedSample + Duplex<f64>>(
    device: Device,
    config: StreamConfig,
    controls: SharedControls,
    sample_tracker: SampleTracker,
) -> anyhow::Result<Stream> {
    let sample_rate = config.sample_rate.0 as f64;
    let mut sound = create_sound(controls);
    sound.set_sample_rate(sample_rate);

    let mut next_value = move || sound.get_stereo();
    let channels = config.channels as usize;
    let err_fn = |err| tracing::error!(?err, "Error on audio stream");
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value, sample_tracker.clone())
            },
            err_fn,
            None,
        )?;

    stream.play()?;

    Ok(stream)
}

/// Callback function to send the current sample to the speakers.
fn write_data<T: SizedSample + Duplex<f64>>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f32, f32),
    sample_tracker: SampleTracker,
) {
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = T::from_sample(sample.0 as f64);
        let right: T = T::from_sample(sample.1 as f64);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                sample_tracker.add_sample(left.to_sample::<f64>());
                *sample = left
            } else {
                *sample = right
            };
        }
    }
}
