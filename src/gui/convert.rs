use crate::gui::TabView;
use ascc::load_asc_from_file;
use atomic_float::{AtomicF32, AtomicF64};
use egui::{Color32, Context, Frame, ProgressBar, RichText, Ui, Widget};
use egui_file::FileDialog;
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{env, thread};

pub struct EdfConverter {
    files: Vec<PathBuf>,
    export_handle: Option<JoinHandle<anyhow::Result<()>>>,
    msg_channel: Option<Receiver<String>>,
    messages: String,
    file_dialog: Option<FileDialog>,
    exp_result: Option<anyhow::Result<()>>,
    progress: Arc<AtomicF32>,
    options: ConversionOptions,
}

#[derive(Default, Clone)]
pub struct ConversionOptions {
    compressed: bool,
}

impl EdfConverter {
    pub fn new(files: Vec<PathBuf>) -> Self {
        EdfConverter {
            files,
            export_handle: None,
            msg_channel: None,
            messages: String::new(),
            file_dialog: None,
            exp_result: None,
            progress: Arc::new(AtomicF32::new(0.)),
            options: ConversionOptions::default(),
        }
    }

    fn import(&mut self, ctx: &Context) {
        let files = self.files.clone();

        let (tx, rx) = channel();
        self.msg_channel = Some(rx);

        let ctx_clone = ctx.clone();

        let progress = self.progress.clone();
        let options = self.options.clone();

        let handle = thread::spawn(move || {
            let step = 1. / (files.len() as f32);
            for file in files {
                let mut asc_name = file.clone();
                asc_name.set_extension("asc");

                tx.send(format!("converting {} to asc", file.display()))?;
                let mut child = Command::new("edf2asc")
                    .arg(file.clone())
                    .arg("-y")
                    .arg("-res")
                    .arg("-vel")
                    .arg("-ftime")
                    .arg("-input")
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();

                let child_stdout = child.stdout.take().expect("no stdout");

                let ctx_clone = ctx_clone.clone();

                // let stdout_thread = thread::spawn(move || {
                //     let stdout_lines = BufReader::new(child_stdout).lines();
                //     for line in stdout_lines {
                //         let line = line.unwrap();
                //         println!("{}", line);
                //         ctx_clone.request_repaint();
                //         tx.send(format!("{line}\n")).unwrap();
                //     }
                // });

                let _ = child.wait()?;
                progress.fetch_add(step / 3., Ordering::Relaxed);

                tx.send(format!("importing {}", asc_name.display()))?;
                let exp = load_asc_from_file(asc_name)?;
                progress.fetch_add(step / 3., Ordering::Relaxed);

                // ciborium::ser::into_writer(&exp, BufWriter::new(out_file))?;
                // serde_json::to_writer(BufWriter::new(out_file), &exp)?;
                // let v = postcard::to_stdvec(&exp)?;
                // BufWriter::new(out_file).write(&v)?;
                let bytes = rkyv::to_bytes::<_, 256>(&exp)?;
                let mut exported_file = file.clone();

                if options.compressed {
                    exported_file.set_extension("dat.archive");
                    let out_file = File::create(&exported_file)?;
                    tx.send(format!("writing to {}", exported_file.display()))?;
                    let mut e = GzEncoder::new(Vec::new(), Compression::fast());
                    e.write_all(&bytes).unwrap();
                    BufWriter::new(out_file).write(&e.finish().unwrap())?;
                } else {
                    exported_file.set_extension("dat");
                    let out_file = File::create(&exported_file)?;
                    tx.send(format!("writing to {}", exported_file.display()))?;
                    BufWriter::new(out_file).write(&bytes)?;
                }

                progress.fetch_add(step / 3., Ordering::Relaxed);
            }

            Ok(())
        });

        self.export_handle = Some(handle);
    }
}

impl TabView for EdfConverter {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Converter");

        ui.checkbox(&mut self.options.compressed, "Compress output");

        Frame::none().fill(Color32::DARK_GRAY).show(ui, |ui| {
            for f in &self.files {
                ui.label(f.display().to_string());
            }
        });

        if ui.button("Add file").clicked() {
            let mut dialog = FileDialog::open_file(None);
            dialog.open();
            self.file_dialog = Some(dialog);
        }

        if let Some(dialog) = &mut self.file_dialog {
            if dialog.show(ui.ctx()).selected() {
                if let Some(file) = dialog.path() {
                    if file.extension().unwrap().to_str().unwrap() == "EDF" {
                        self.files.push(file);
                    }
                }
            }
        }

        if ui.button("Import").clicked() {
            self.import(ui.ctx());
        }
        ui.label(&self.messages);
        ProgressBar::new(self.progress.load(Ordering::Relaxed))
            .desired_width(300.0)
            .show_percentage()
            .animate(true)
            .ui(ui);
        if let Some(handle) = self.export_handle.as_ref() {
            if let Ok(msg) = self.msg_channel.as_ref().unwrap().try_recv() {
                self.messages.push_str(&format!("{msg}\n"));
            }
            if handle.is_finished() {
                let res = self
                    .export_handle
                    .take()
                    .unwrap()
                    .join()
                    .expect("should be closed");
                self.exp_result = Some(res);
            }
        } else {
            if let Some(res) = &self.exp_result {
                if let Err(e) = res {
                    ui.label(RichText::new(e.to_string()).color(Color32::RED));
                } else {
                    ui.label("Done!");
                }
            }
        }
    }
}
