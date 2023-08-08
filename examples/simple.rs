use ascc::load_asc_from_file_with_progress;
use std::path::PathBuf;

pub fn main() -> anyhow::Result<()> {
    let exp = load_asc_from_file_with_progress(PathBuf::from("resources/Patterns_214_0.asc"))?;
    for (i, t) in exp.trials.iter().enumerate() {
        println!("trial {i}, events: {}", t.events.len());
        for e in &t.events {
            println!("\t{:?}", e);
        }
    }
    Ok(())
}
