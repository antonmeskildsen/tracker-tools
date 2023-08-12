use crate::gui::plots::create_line;
use crate::gui::TabView;
use ascc::generic::Experiment;
use egui::{plot, Color32, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use std::iter::zip;

pub struct ExperimentViewer {
    pub id: String,
    pub exp: Experiment,
    pub open_trials: Vec<bool>,
    pub current_trial: i32,
    pub show_metadata: bool,
    pub show_variables: bool,
    pub plot_options: PlotOptions,
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

impl ExperimentViewer {
    pub fn new(exp: Experiment, id: String) -> Self {
        let open_trials = vec![false; exp.trials.len()];
        ExperimentViewer {
            id,
            exp,
            open_trials,
            current_trial: 0,
            show_metadata: false,
            show_variables: false,
            plot_options: PlotOptions::default(),
        }
    }
}

impl TabView for ExperimentViewer {
    fn ui(&mut self, ui: &mut Ui) {
        let current_trial = self.exp.trials[self.current_trial as usize].clone();

        egui::SidePanel::left(format!("left_panel_{}", &self.id)).show_inside(ui, |ui| {
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

        egui::SidePanel::right(format!("right_panel_{}", &self.id)).show_inside(ui, |ui| {
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

        egui::CentralPanel::default().show_inside(ui, |ui| {
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
            egui::Window::new("Metadata").show(ui.ctx(), |ui| {
                for l in &self.exp.meta.preamble_lines {
                    ui.label(l);
                }
            });
        }

        egui::Window::new("Trial variables")
            .open(&mut self.show_variables)
            .default_size([600., 400.])
            .vscroll(true)
            .show(ui.ctx(), |ui| {
                TableBuilder::new(ui)
                    .column(Column::exact(50.))
                    .columns(
                        Column::auto().at_least(200.),
                        self.exp.variable_labels.len(),
                    )
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("Variable");
                        });
                        for name in &self.exp.variable_labels {
                            header.col(|ui| {
                                ui.label(name);
                            });
                        }
                    })
                    .body(|mut body| {
                        for (i, trial) in self.exp.trials.iter().enumerate() {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("trial {i}"));
                                });
                                for i in 0..self.exp.variable_labels.len() {
                                    row.col(|ui| {
                                        ui.label(&trial.variables[i]);
                                    });
                                }
                            });
                        }
                    });
            });
    }
}
