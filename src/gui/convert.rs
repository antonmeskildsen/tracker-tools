use crate::gui::TabView;
use ascc::load_asc_from_file;
use egui::Ui;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::Command;
use std::thread::JoinHandle;
use std::{env, thread};

pub struct EdfConverter {
    files: Vec<PathBuf>,
    export_handle: Option<JoinHandle<anyhow::Result<()>>>,
}

impl EdfConverter {
    pub fn new(files: Vec<PathBuf>) -> Self {
        EdfConverter {
            files,
            export_handle: None,
        }
    }

    fn export(&mut self) {
        let file_name = self.files[0].clone();
        let handle = thread::spawn(move || {
            let mut asc_name = file_name.clone();
            asc_name.set_extension(".asc");
            let mut json_name = file_name.clone();
            json_name.set_extension(".json");

            let s = Command::new("edf2asc").arg(file_name).status()?;

            let exp = load_asc_from_file(asc_name)?;

            let out_file = File::create(json_name)?;
            serde_json::to_writer(BufWriter::new(out_file), &exp)?;

            Ok(())
        });
        self.export_handle = Some(handle);
    }
}

impl TabView for EdfConverter {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Converter");
        if ui.button("Export").clicked() {
            self.export();

            if let Some(handle) = self.export_handle.as_ref() {
                if !handle.is_finished() {
                    ui.spinner();
                }
            }
        }
    }
}
