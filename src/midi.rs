use fundsp::math::midi_hz;
use midi_msg::{ChannelVoiceMsg, MidiMsg};
use midir::{Ignore, MidiInput, MidiInputPort};
use read_input::{InputBuild, prelude::input};

use crate::SharedControls;

/// Finds connected MIDI devices and takes the first one.
pub fn get_midi_device(midi_input: &mut MidiInput) -> anyhow::Result<MidiInputPort> {
    midi_input.ignore(Ignore::None);
    let input_ports = midi_input.ports();
    if input_ports.is_empty() {
        anyhow::bail!("No MIDI devices attached")
    } else {
        tracing::info!(
            port_name=midi_input.port_name(&input_ports[0])?,
            "Using MIDI device",
        );
        Ok(input_ports[0].clone())
    }
}

/// Connects to the MIDI input port and listens for MIDI messages.
/// Updates the shared controls when messages are received.
pub fn run_listener(
    midi_input: MidiInput,
    input_port: MidiInputPort,
    controls: SharedControls,
) -> anyhow::Result<()> {
    tracing::info!("Opening MIDI connection");
    let input_port_name = midi_input.port_name(&input_port)?;
    let _input_conn = midi_input.connect(
        &input_port,
        "midir-read-input",
        move |_stamp, message, _| {
            let (msg, _len) = match MidiMsg::from_midi(message) {
                Ok(value) => value,
                Err(err) => {
                    tracing::error!(?err, "Failed to parse MIDI message");
                    return;
                }
            };

            if let MidiMsg::ChannelVoice { channel: _, msg } = msg {
                tracing::info!(?msg, "Received");
                match msg {
                    ChannelVoiceMsg::NoteOn { note, velocity } => {
                        controls.pitch.set_value(midi_hz(note as f32));
                        controls.volume.set_value(velocity as f32 / 127.0);
                        controls.pitch_bend.set_value(1.0);
                        controls.control.set_value(1.0);
                    }
                    ChannelVoiceMsg::NoteOff { note, velocity: _ } => {
                        if controls.pitch.value() == midi_hz(note as f32) {
                            controls.control.set_value(-1.0);
                        }
                    }
                    ChannelVoiceMsg::PitchBend { bend } => {
                        controls
                            .pitch_bend
                            .set_value(pitch_bend_factor(bend) as f32);
                    }
                    _ => {}
                }
            }
        },
        (),
    )?;

    tracing::info!(input_port_name, "MIDI connection open");

    let _ = input::<String>().msg("(press enter to exit)...\n").get();

    tracing::info!("Closing MIDI connection");

    Ok(())
}

fn pitch_bend_factor(bend: u16) -> f64 {
    2.0_f64.powf(((bend as f64 - 8192.0) / 8192.0) / 12.0)
}
