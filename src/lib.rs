use crate::asc::Element;
use crate::generic::Experiment;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Context;

#[cfg(feature = "py-ext")]
use serde::{Serialize, Deserialize};

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

pub fn asc_to_generic_with_progress(input: &str) -> anyhow::Result<Experiment> {
    let lines: Vec<(usize, &str)> = input.lines().enumerate().collect();
    let res: anyhow::Result<Vec<Element>> = lines
        .into_par_iter()
        .progress()
        .map(|(i, e)| Element::from_str(e).with_context(|| format!("at line {i}, content: {e}")))
        .collect();
    Ok(Experiment::from(res?))
}

pub fn asc_to_generic(input: &str) -> anyhow::Result<Experiment> {
    let lines: Vec<(usize, &str)> = input.lines().enumerate().collect();
    let res: anyhow::Result<Vec<Element>> = lines
        .into_par_iter()
        .map(|(i, e)| Element::from_str(e).with_context(|| format!("at line {i}, content: {e}")))
        .collect();
    Ok(Experiment::from(res?))
}

pub fn load_asc_from_file_with_progress(path: PathBuf) -> anyhow::Result<Experiment> {
    let s = std::fs::read_to_string(path)?;
    asc_to_generic_with_progress(&s)
}

pub fn load_asc_from_file(path: PathBuf) -> anyhow::Result<Experiment> {
    let s = std::fs::read_to_string(path)?;
    asc_to_generic(&s)
}
