use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use eframe::Frame;
use egui::{Color32, Context, RichText, Ui, Vec2, WidgetText};
use egui_dock::{DockArea, NodeIndex, Style, TabViewer, Tree};
use egui_file::FileDialog;
use flate2::write::{GzDecoder, ZlibDecoder};

use crate::gui::convert::EdfConverter;
use ascc::generic::{Experiment, Trial};
use ascc::load_asc_from_file;

use crate::gui::experiment_viewer::ExperimentViewer;
use crate::gui::home::HomeView;

mod convert;
mod experiment_viewer;
mod home;
mod plots;

pub fn run() -> eframe::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(1280., 800.));
    eframe::run_native(
        "viscom gui",
        native_options,
        Box::new(|cc| Box::new(TrackerToolsApp::new())),
    )
}

#[derive(Default)]
pub struct TrackerToolsApp {
    tabs: Tree<Tab>,
    file_dialog: Option<FileDialog>,
    opened_file: Option<PathBuf>,
    status: AppStatus,
}

#[derive(Default)]
pub enum AppStatus {
    #[default]
    None,
    Msg(String),
    Err(String),
}

impl TrackerToolsApp {
    fn new() -> Self {
        TrackerToolsApp {
            tabs: Tree::new(vec![Tab::new("Home", HomeView)]),
            ..Default::default()
        }
    }
}

impl eframe::App for TrackerToolsApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let mut dialog = FileDialog::open_file(self.opened_file.clone());
                        dialog.open();
                        self.file_dialog = Some(dialog);
                    }
                });
                ui.menu_button("Tools", |ui| {
                    if ui.button("Converter").clicked() {
                        self.tabs
                            .push_to_focused_leaf(Tab::new("converter", EdfConverter::new(vec![])));
                    }
                })
            });
        });

        if let Some(dialog) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    let ext = file.extension().unwrap().to_str().unwrap();
                    let title = file.file_name().unwrap().to_str().unwrap();
                    match ext {
                        "asc" => {
                            self.opened_file = Some(file.clone());

                            let exp = load_asc_from_file(file.clone()).unwrap();

                            self.tabs.push_to_first_leaf(Tab::new(
                                title,
                                ExperimentViewer::new(exp.clone(), title.to_string()),
                            ));
                        }
                        "EDF" | "edf" => {
                            self.tabs.push_to_first_leaf(Tab::new(
                                title,
                                EdfConverter::new(vec![file.clone()]),
                            ));
                        }
                        "cbor" => {
                            let base = File::open(&file).unwrap();

                            let exp = ciborium::de::from_reader(BufReader::new(base)).unwrap();
                            self.tabs.push_to_first_leaf(Tab::new(
                                title,
                                ExperimentViewer::new(exp, title.to_string()),
                            ));
                        }
                        "json" => {
                            let base = File::open(&file).unwrap();

                            let exp = serde_json::from_reader(BufReader::new(base)).unwrap();
                            self.tabs.push_to_first_leaf(Tab::new(
                                title,
                                ExperimentViewer::new(exp, title.to_string()),
                            ));
                        }
                        "pc" => {
                            let base = File::open(&file).unwrap();

                            let mut bytes = Vec::new();
                            BufReader::new(base).read_to_end(&mut bytes).unwrap();
                            let exp = postcard::from_bytes(bytes.as_slice()).unwrap();
                            self.tabs.push_to_first_leaf(Tab::new(
                                title,
                                ExperimentViewer::new(exp, title.to_string()),
                            ));
                        }
                        "dat" => {
                            let base = File::open(&file).unwrap();

                            let mut bytes = Vec::new();
                            BufReader::new(base).read_to_end(&mut bytes).unwrap();
                            let exp = rkyv::from_bytes(&bytes).unwrap();
                            self.tabs.push_to_first_leaf(Tab::new(
                                title,
                                ExperimentViewer::new(exp, title.to_string()),
                            ));
                        }
                        "archive" => {
                            let base = File::open(&file).unwrap();

                            let mut bytes = Vec::new();
                            BufReader::new(base).read_to_end(&mut bytes).unwrap();
                            let mut e = GzDecoder::new(Vec::new());
                            e.write_all(&mut bytes).unwrap();
                            let exp = rkyv::from_bytes(e.finish().unwrap().as_slice()).unwrap();
                            self.tabs.push_to_first_leaf(Tab::new(
                                title,
                                ExperimentViewer::new(exp, title.to_string()),
                            ));
                        }
                        _ => self.status = AppStatus::Err(format!("invalid file extension {ext}")),
                    }
                }
            }
        }

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| match &self.status {
            AppStatus::None => {}
            AppStatus::Msg(s) => {
                ui.label(s);
            }
            AppStatus::Err(e) => {
                ui.label(RichText::new(e).color(Color32::RED));
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.tabs)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut ETabViewer {});
        });
    }
}

pub struct TrialWindow {
    pub trial: Trial,
}

pub struct ETabViewer;

pub struct Tab {
    title: String,
    view: Box<dyn TabView>,
}

impl Tab {
    pub fn new<S: Into<String>, T: TabView + 'static>(title: S, view: T) -> Self {
        Tab {
            title: title.into(),
            view: Box::new(view),
        }
    }
}

pub trait TabView {
    fn ui(&mut self, ui: &mut Ui);
}

impl TabViewer for ETabViewer {
    type Tab = Tab;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.push_id(&tab.title, |ui| {
            tab.view.ui(ui);
        });
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        (&tab.title).into()
    }
}
