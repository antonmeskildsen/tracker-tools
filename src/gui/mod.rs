use std::path::PathBuf;

use eframe::Frame;
use egui::{Color32, Context, RichText, Ui, WidgetText};
use egui_dock::{DockArea, Style, TabViewer, Tree};
use egui_file::FileDialog;

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
    native_options.maximized = true;
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
