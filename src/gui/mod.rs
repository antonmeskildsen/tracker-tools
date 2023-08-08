mod plots;

use crate::gui::plots::create_line;
use ascc::generic::{Experiment, Trial};
use ascc::load_asc_from_file;
use eframe::Frame;
use egui::widgets::plot;
use egui::{Color32, Context, RichText, ScrollArea};
use egui_extras::{Column, TableBuilder};
use egui_file::FileDialog;
use std::iter::zip;
use std::path::PathBuf;

pub fn run(exp: Experiment) -> eframe::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.maximized = true;
    eframe::run_native(
        "viscom gui",
        native_options,
        Box::new(|cc| Box::new(AscViewerApp::new(exp))),
    )
}

pub struct AscViewerApp {
    pub exp: Experiment,
    pub open_trials: Vec<bool>,
    pub current_trial: i32,
    pub show_metadata: bool,
    pub show_variables: bool,
    pub plot_options: PlotOptions,
    pub file_dialog: Option<FileDialog>,
    pub opened_file: Option<PathBuf>,
}

#[derive(Default)]
pub struct PlotOptions {
    pub left_x: bool,
    pub left_y: bool,
    pub right_x: bool,
    pub right_y: bool,
    pub velocity_left: bool,
    pub velocity_right: bool,
}

impl AscViewerApp {
    pub fn new(exp: Experiment) -> Self {
        let open_trials = vec![false; exp.trials.len()];
        AscViewerApp {
            exp,
            open_trials,
            current_trial: 0,
            show_metadata: false,
            show_variables: false,
            plot_options: PlotOptions::default(),
            file_dialog: None,
            opened_file: None,
        }
    }
}

impl eframe::App for AscViewerApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let current_trial = self.exp.trials[self.current_trial as usize].clone();

        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let mut dialog = FileDialog::open_file(self.opened_file.clone());
                        dialog.open();
                        self.file_dialog = Some(dialog);
                    }
                });
            });
        });

        if let Some(dialog) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.clone());

                    self.exp = load_asc_from_file(file).unwrap();
                }
            }
        }

        egui::SidePanel::left("trials").show(ctx, |ui| {
            ui.vertical_centered(|ui| ui.heading("Trials:"));
            for (i, trial) in self.exp.trials.iter().enumerate() {
                ui.selectable_value(
                    &mut self.current_trial,
                    i as i32,
                    format!("trial {}", trial.id),
                );
            }

            ui.separator();
            ui.heading(RichText::new("Tasks").size(20.0));

            ui.toggle_value(&mut self.show_metadata, "Metadata");
            ui.toggle_value(&mut self.show_variables, "Trial variables");
            // should be menu button
        });

        egui::SidePanel::right("info").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("Inspector for trial {}", self.current_trial));
            });

            ui.separator();
            ui.label("Overview");
            egui::Grid::new("overview_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("events:");
                    ui.label(current_trial.events.len().to_string());
                    ui.end_row();

                    ui.label("samples:");
                    ui.label(current_trial.samples.len().to_string());
                    ui.end_row();

                    ui.label("raw samples:");
                    ui.label(current_trial.raw_samples.len().to_string());
                    ui.end_row();
                });
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            plot::Plot::new(format!("view{}", self.current_trial))
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    if self.plot_options.left_x {
                        let line_left_x = create_line(
                            &current_trial.samples,
                            |x| Some(x.time),
                            |y| Some(y.left?.position[0]),
                        )
                        .color(Color32::RED);
                        plot_ui.line(line_left_x);
                    }

                    if self.plot_options.right_x {
                        let line_right_x = create_line(
                            &current_trial.samples,
                            |x| Some(x.time),
                            |y| Some(y.right?.position[0]),
                        )
                        .color(Color32::LIGHT_RED);
                        plot_ui.line(line_right_x);
                    }

                    if self.plot_options.left_y {
                        let line = create_line(
                            &current_trial.samples,
                            |x| Some(x.time),
                            |y| Some(y.left?.position[1]),
                        )
                        .color(Color32::BLUE);
                        plot_ui.line(line);
                    }

                    if self.plot_options.right_y {
                        let line = create_line(
                            &current_trial.samples,
                            |x| Some(x.time),
                            |y| Some(y.right?.position[1]),
                        )
                        .color(Color32::LIGHT_BLUE);
                        plot_ui.line(line);
                    }
                });

            ui.checkbox(&mut self.plot_options.left_x, "left eye x");
            ui.checkbox(&mut self.plot_options.right_x, "right eye x");
            ui.checkbox(&mut self.plot_options.left_y, "left eye y");
            ui.checkbox(&mut self.plot_options.right_y, "right eye y");
        });

        if self.show_metadata {
            egui::Window::new("Metadata").show(ctx, |ui| {
                for l in &self.exp.meta.preamble_lines {
                    ui.label(l);
                }
            });
        }

        egui::Window::new("Trial variables")
            .open(&mut self.show_variables)
            .default_size([600., 400.])
            .show(ctx, |ui| {
                TableBuilder::new(ui)
                    .column(Column::auto())
                    .columns(Column::auto(), self.exp.variable_labels.len())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Variable");
                        });
                        for i in 0..self.exp.variable_labels.len() {
                            header.col(|ui| {
                                ui.heading(format!("trial {i}"));
                            });
                        }
                    })
                    .body(|mut body| {
                        for (name, trial) in zip(&self.exp.variable_labels, &self.exp.trials) {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(name);
                                });
                                for val in &trial.variables {
                                    row.col(|ui| {
                                        ui.label(val);
                                    });
                                }
                            });
                        }
                    });
            });
    }
}

pub struct TrialWindow {
    pub trial: Trial,
}
