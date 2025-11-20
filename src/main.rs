#![allow(clippy::precedence)]

use fundsp::hacker::{Shared, shared};
use midir::MidiInput;

use crate::sample_tracker::SampleTracker;

mod audio;
mod midi;
mod sample_tracker;
mod ui;

/// Shared fundsp controls, the produced audio is entirely controlled by these values.
/// The MIDI input is responsible for setting the value of these controls.
#[derive(Clone)]
pub struct SharedControls {
    pub pitch: Shared,
    pub volume: Shared,
    pub pitch_bend: Shared,
    pub control: Shared,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let mut midi_input = MidiInput::new("midir reading input")?;
    let input_port = midi::get_midi_device(&mut midi_input)?;

    let controls = SharedControls {
        pitch: shared(0.0),
        volume: shared(0.0),
        pitch_bend: shared(1.0),
        control: shared(0.0),
    };

    let sample_tracker = SampleTracker::default();

    // Connect to the default audio device and start streaming audio
    let audio_controls = controls.clone();
    let audio_sample_tracker = sample_tracker.clone();
    std::thread::spawn(move || {
        let _stream = match audio::run_on_default_device(audio_controls, audio_sample_tracker) {
            Ok(stream) => stream,
            Err(err) => {
                tracing::error!(?err, "Audio error");
                std::process::exit(1);
            }
        };
        // Keep this thread alive indefinitely
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    // Start the MIDI input that will receive events and update the controls accordingly
    let midi_controls = controls.clone();
    std::thread::spawn(move || {
        if let Err(err) = midi::run_listener(midi_input, input_port, midi_controls) {
            tracing::error!(?err, "MIDI error");
            std::process::exit(1);
        }
    });

    // Start the UI on the main thread, when the window is closed the other threads will be stopped
    // and the application will exit
    ui::run(
        controls.clone(),
        audio::create_sound(controls),
        sample_tracker,
    )
}
