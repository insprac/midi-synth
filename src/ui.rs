use fundsp::hacker::AudioUnit;

use crate::{
    SharedControls,
    sample_tracker::{self, SampleTracker},
};

pub fn run(
    controls: SharedControls,
    audio: Box<dyn AudioUnit>,
    sample_tracker: SampleTracker,
) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600., 400.]),
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
}

impl App {
    pub fn new(
        controls: SharedControls,
        audio: Box<dyn AudioUnit>,
        sample_tracker: SampleTracker,
    ) -> Self {
        Self {
            controls,
            audio,
            sample_tracker,
        }
    }

    fn wave_datapoints(&mut self) -> Vec<[f64; 2]> {
        self.sample_tracker.samples_vec().into_iter().enumerate()
            .map(|(i, sample)| [i as f64, sample])
            .collect()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Synth");

            egui_plot::Plot::new("Wave").show(ui, |plot_ui| {
                let line_points = egui_plot::PlotPoints::from_iter(self.wave_datapoints());
                let line = egui_plot::Line::new("Wave", line_points);
                plot_ui.line(line);
            });
        });

        // Continuous redraw for animated plots
        ctx.request_repaint();
    }
}
