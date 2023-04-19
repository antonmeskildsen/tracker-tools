use ascc::generic::Experiment;
use polars::prelude::*;
use std::fs::File;
use std::io::BufReader;

pub fn main() -> anyhow::Result<()> {
    let file = File::open("test_files/out.cbor")?;
    let bf = BufReader::new(file);

    println!("Reading file");
    let data: Experiment = ciborium::de::from_reader(bf)?;

    println!("Extracting variables");
    let mut df = data.trials[0].samples()?;

    println!("{}", df);

    let out_file = File::create("test_files/smp.parquet")?;
    ParquetWriter::new(out_file).finish(&mut df)?;

    Ok(())
}
