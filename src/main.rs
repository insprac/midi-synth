#![allow(clippy::precedence)]

use fundsp::hacker::{Shared, shared};
use midir::{MidiInput};

mod midi;
mod audio;

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

    // Connect to the default audio device and start streaming audio (on it's own thread)
    audio::run_on_default_device(controls.clone())?;

    // Start the MIDI input that will receive events and update the controls accordingly
    midi::run_listener(midi_input, input_port, controls)
}
