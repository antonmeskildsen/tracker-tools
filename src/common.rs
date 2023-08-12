use anyhow::anyhow;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(feature = "py-ext")]
use pyo3::pyclass;
use rkyv::Archive;

#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Serialize, Deserialize, Debug, Clone, Copy,
)]
#[archive(check_bytes)]
#[cfg_attr(feature = "py-ext", pyclass)]
pub enum Eye {
    Left,
    Right,
}

impl FromStr for Eye {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            _ => Err(anyhow!(format!("Invalid eye specification string: {s}"))),
        }
    }
}
