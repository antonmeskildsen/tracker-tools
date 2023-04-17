mod asc;

use std::str::FromStr;
use anyhow::Result;
use rust_decimal::Decimal;
use crate::asc::BinocularSample;

fn main() -> Result<()> {
    let s = std::fs::read_to_string("src/test.asc")?;

    let samples: Vec<BinocularSample> = Vec::new();

    for l in s.lines() {
        let elements: Vec<&str> = l.split_whitespace().collect();
        if elements.len() > 0 {
            if let Ok(time) = Decimal::from_str(elements[0]) {
                let plx = Decimal::from_str(elements[1]).ok();
                let ply = Decimal::from_str(elements[2]).ok();
                let pla = Decimal::from_str(elements[3]).ok();
                let prx = Decimal::from_str(elements[4]).ok();
                let pry = Decimal::from_str(elements[5]).ok();
                let pra = Decimal::from_str(elements[6]).ok();

            }
        }
    }

    Ok(())
}
