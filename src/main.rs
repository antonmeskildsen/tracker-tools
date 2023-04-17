mod asc;
pub mod generic;

use crate::asc::Element;
use crate::generic::GazeData;
use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rayon::prelude::*;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::BufWriter;
use std::str::FromStr;

fn main() -> Result<()> {
    let s = std::fs::read_to_string("test_files/SmoothPursuits_128_2.asc")?;

    let lines: Vec<&str> = s.lines().collect();
    let nlines = lines.len();

    let res: Result<Vec<Element>> = lines
        .into_par_iter()
        .progress()
        .map(Element::from_str)
        .collect();
    let gd = GazeData::from(res?);

    for t in &gd.trials {
        println!("Trial {}, n-samples: {}", t.id, t.samples.len());
    }

    let out = File::create("test_files/out.cbor")?;
    let wr = BufWriter::new(out);
    ciborium::ser::into_writer(&gd, wr)?;
    // serde_json::to_writer(wr, &gd)?;
    Ok(())
}
