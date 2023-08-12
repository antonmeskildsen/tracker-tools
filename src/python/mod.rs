mod generic;

#[cfg(feature = "dataframes")]
mod export;

use crate::generic::{
    EventInfo, Experiment, MetaData, RawSample, Sample, TargetInfo, TimeRecord, Trial,
};
use crate::{Decimal, NaiveDateTime};
use chrono::{Datelike, ParseResult, Timelike};
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDict, PyList, PyString};
use rust_decimal::prelude::ToPrimitive;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::ops::Sub;
use std::path::PathBuf;
use std::str::FromStr;

#[pyfunction]
fn load_asc_from_file(path: PathBuf) -> PyResult<Experiment> {
    let exp = crate::load_asc_from_file_with_progress(path)?;
    Ok(exp)
}

#[pyfunction]
fn load_experiment_file(path: PathBuf) -> PyResult<Experiment> {
    let base = File::open(&path)?;

    let mut bytes = Vec::new();
    BufReader::new(base).read_to_end(&mut bytes)?;
    Ok(rkyv::from_bytes(&bytes).unwrap())
}

#[pymodule]
fn ascc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Experiment>()?;
    m.add_class::<Trial>()?;
    m.add_class::<MetaData>()?;
    m.add_class::<TimeRecord>()?;
    m.add_class::<TargetInfo>()?;
    m.add_class::<Sample>()?;
    m.add_class::<RawSample>()?;

    m.add_function(wrap_pyfunction!(load_asc_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(load_experiment_file, m)?)?;

    Ok(())
}

impl Display for Decimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl ToPyObject for NaiveDateTime {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyDateTime::new(
            py,
            self.0.year(),
            self.0.month() as u8,
            self.0.day() as u8,
            self.0.hour() as u8,
            self.0.minute() as u8,
            self.0.second() as u8,
            0,
            None,
        )
        .unwrap()
        .to_object(py)
    }
}

impl IntoPy<Py<PyAny>> for NaiveDateTime {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        PyDateTime::new(
            py,
            self.0.year(),
            self.0.month() as u8,
            self.0.day() as u8,
            self.0.hour() as u8,
            self.0.minute() as u8,
            self.0.second() as u8,
            0,
            None,
        )
        .unwrap()
        .into_py(py)
    }
}

impl ToPyObject for Decimal {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.0.to_f64().unwrap_or_default().into_py(py)
    }
}

impl IntoPy<PyObject> for Decimal {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.0.to_f64().unwrap_or_default().into_py(py)
    }
}

impl ToPyObject for EventInfo {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            EventInfo::Fixation {
                average_position,
                average_pupil_area,
            } => {
                let pos: PyObject = ("average_position", average_position.to_vec()).into_py(py);
                let area = ("average_pupil_area", average_pupil_area.into_py(py)).into_py(py);
                let seq = PyList::new(py, [pos, area]).to_object(py);
                PyDict::from_sequence(py, seq).unwrap().to_object(py)
            }
            EventInfo::Saccade {
                start_position,
                end_position,
                movement_angle,
                peak_velocity,
            } => {
                let start_position: PyObject =
                    ("start_position", start_position.map(|p| p.to_vec())).into_py(py);
                let end_position = ("end_position", end_position.map(|p| p.to_vec())).into_py(py);
                let movement_angle =
                    ("movement_angle", movement_angle.map(|a| a.into_py(py))).into_py(py);
                let peak_velocity = ("peak_velocity", peak_velocity.into_py(py)).into_py(py);
                let seq = PyList::new(
                    py,
                    [start_position, end_position, movement_angle, peak_velocity],
                )
                .to_object(py);
                PyDict::from_sequence(py, seq).unwrap().to_object(py)
            }
            EventInfo::Blink => PyString::new(py, "blink").to_object(py),
        }
    }
}

impl IntoPy<PyObject> for EventInfo {
    fn into_py(self, _py: Python<'_>) -> PyObject {
        todo!()
    }
}

impl FromStr for Decimal {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(rust_decimal::Decimal::from_str(s)?))
    }
}

impl Decimal {
    pub fn from_scientific(s: &str) -> Result<Self, rust_decimal::Error> {
        Ok(Self(rust_decimal::Decimal::from_scientific(s)?))
    }
}

impl Sub for Decimal {
    type Output = Decimal;

    fn sub(self, rhs: Self) -> Self::Output {
        Decimal(self.0 - rhs.0)
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl NaiveDateTime {
    pub fn parse_from_str(s: &str, fmt: &str) -> ParseResult<Self> {
        Ok(NaiveDateTime(chrono::NaiveDateTime::parse_from_str(
            s, fmt,
        )?))
    }
}
