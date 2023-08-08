mod gui;

use anyhow::Result;
use ascc::asc::Element;
use ascc::generic::Experiment;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::str::FromStr;

use ascc::{load_asc_from_file, load_asc_from_file_with_progress};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Convert {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    Gui,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Convert { input, output: _ } => {
            let s = std::fs::read_to_string("test_files/Reaction_144_0.asc")?;

            let lines: Vec<&str> = s.lines().collect();
            let _nlines = lines.len();

            let res: Result<Vec<Element>> = lines
                .into_par_iter()
                .progress()
                .map(Element::from_str)
                .collect();
            let gd = Experiment::from(res?);

            for t in &gd.trials {
                let t_lens: HashMap<String, usize> = t
                    .targets
                    .iter()
                    .map(|(name, info)| (name.clone(), info.len()))
                    .collect();
                println!(
                    "Trial {}, n-samples: {}, variables: {:?}, targets: {:?}",
                    t.id,
                    t.samples.len(),
                    t.variables,
                    t_lens,
                );
            }

            let out = File::create("test_files/out.cbor")?;
            let wr = BufWriter::new(out);
            ciborium::ser::into_writer(&gd, wr)?;
        }
        Commands::Gui => {
            gui::run().expect("error");
        }
    }

    Ok(())
}
