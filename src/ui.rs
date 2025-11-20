use fundsp::hacker::AudioUnit;
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::sync::Arc;

use crate::{
    SharedControls,
    sample_tracker::{self, SampleTracker},
};

const SAMPLE_RATE: f32 = 44100.;

pub fn run(
    controls: SharedControls,
    audio: Box<dyn AudioUnit>,
    sample_tracker: SampleTracker,
) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000., 800.]),
        ..Default::default()
    };

    eframe::run_native(
        "MIDI Synth",
        options,
        Box::new(|_| Ok(Box::new(App::new(controls, audio, sample_tracker)))),
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))
}

pub struct App {
    controls: SharedControls,
    audio: Box<dyn AudioUnit>,
    sample_tracker: SampleTracker,
    fft: Arc<dyn Fft<f32>>,
}

impl App {
    pub fn new(
        controls: SharedControls,
        audio: Box<dyn AudioUnit>,
        sample_tracker: SampleTracker,
    ) -> Self {
        let mut fft_planner = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(SampleTracker::BUFFER_SIZE);

        Self {
            controls,
            audio,
            sample_tracker,
            fft,
        }
    }

    fn prepare_wave_datapoints(&self, samples: &[f64]) -> Vec<[f64; 2]> {
        samples
            .iter()
            .enumerate()
            .map(|(i, sample)| [i as f64, *sample])
            .collect()
    }

    fn prepare_fft_datapoints(&self, samples: &[f64]) -> Vec<[f64; 2]> {
        let mut fft_buffer: Vec<Complex<f32>> = (0..SampleTracker::BUFFER_SIZE)
            .map(|i| Complex {
                re: samples.get(i).copied().unwrap_or(0.) as f32,
                im: 0.,
            })
            .collect();

        self.fft.process(&mut fft_buffer);

        fft_buffer
            .iter()
            .take(SampleTracker::BUFFER_SIZE / 2)
            .enumerate()
            .map(|(i, complex)| {
                let frequency = i as f32 * SAMPLE_RATE / SampleTracker::BUFFER_SIZE as f32;
                let magnitude = complex.norm();
                [frequency as f64, magnitude as f64]
            })
            .collect()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let samples = self.sample_tracker.samples_vec();

            let available_height = ui.available_height();
            let plot_height = available_height / 2.0 - ui.spacing().item_spacing.y;

            egui_plot::Plot::new("Wave").height(plot_height).show(ui, |plot_ui| {
                let datapoints = self.prepare_wave_datapoints(&samples);
                let line_points = egui_plot::PlotPoints::from_iter(datapoints);
                let line = egui_plot::Line::new("Wave", line_points);
                plot_ui.line(line);
            });

            egui_plot::Plot::new("FFT").height(plot_height).show(ui, |plot_ui| {
                let datapoints = self.prepare_fft_datapoints(&samples);
                let line_points = egui_plot::PlotPoints::from_iter(datapoints);
                let line = egui_plot::Line::new("FFT", line_points);
                plot_ui.line(line);
            });
        });

        // Continuous redraw for animated plots
        ctx.request_repaint();
    }
}
