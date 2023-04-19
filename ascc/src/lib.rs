use crate::asc::Element;
use crate::generic::Experiment;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

pub mod asc;
pub mod common;
pub mod generic;

#[cfg(feature = "dataframes")]
pub mod export;

#[cfg(feature = "py-ext")]
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Decimal(rust_decimal::Decimal);

#[cfg(not(feature = "py-ext"))]
pub type Decimal = rust_decimal::Decimal;

#[cfg(feature = "py-ext")]
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct NaiveDateTime(chrono::NaiveDateTime);

#[cfg(not(feature = "py-ext"))]
pub type NaiveDateTime = chrono::NaiveDateTime;

#[cfg(feature = "py-ext")]
pub mod python;

pub fn asc_to_generic(input: &str) -> anyhow::Result<Experiment> {
    let lines: Vec<&str> = input.lines().collect();
    let res: anyhow::Result<Vec<Element>> = lines
        .into_par_iter()
        .progress()
        .map(Element::from_str)
        .collect();
    Ok(Experiment::from(res?))
}

pub fn load_asc_from_file(path: PathBuf) -> anyhow::Result<Experiment> {
    let s = std::fs::read_to_string(path)?;
    asc_to_generic(&s)
}
