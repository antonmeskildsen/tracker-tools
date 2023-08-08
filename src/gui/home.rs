use crate::gui::TabView;
use egui::Ui;

pub struct HomeView;

impl TabView for HomeView {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Home");
    }
}
