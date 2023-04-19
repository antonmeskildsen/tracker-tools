// use crate::generic::{EyeSampleData, Sample, Trial, Vector};
// use rust_decimal::Decimal;

// #[macro_export]
// macro_rules! extract {
//     ( $col:item, $prop:item ) => {
//         $col.iter().map(|i| i.$prop).collect()
//     };
// }
//
// fn get_opt<T, U, F: FnMut(&T) -> U>(collection: &Vec<T>, f: F) -> Vec<U> {
//     collection.iter().map(f).collect()
// }
//
// impl Trial {
//     fn sample_opt<U, F: FnMut(&Sample) -> U>(&self, f: F) -> Vec<U> {
//         get_opt(&self.samples, f)
//     }
//
//     pub fn samples_time(&self) -> Vec<Decimal> {
//         self.sample_opt(|s| s.time)
//     }
//
//     pub fn samples_left(&self) -> Vec<Option<EyeSampleData>> {
//         self.sample_opt(|s| s.left)
//     }
//
//     pub fn samples_right(&self) -> Vec<Option<EyeSampleData>> {
//         self.sample_opt(|s| s.right)
//     }
//
//     pub fn samples_resolution(&self) -> Vec<Option<Vector>> {
//         self.sample_opt(|s| s.resolution)
//     }
// }
//
// impl EyeSampleData {
//     pub fn
// }

use crate::generic::{Experiment, Trial};
use crate::Decimal;
use polars::export::chrono::NaiveTime;
use polars::prelude::AnyValue;
use polars::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use std::collections::{HashMap, HashSet};

impl Experiment {
    pub fn trial_variables(&self) -> PolarsResult<DataFrame> {
        let vars = &self.variable_labels;

        let mut var_values: Vec<Vec<String>> = vec![Vec::new(); vars.len()];

        for t in &self.trials {
            t.variables
                .iter()
                .zip(&mut var_values)
                .for_each(|(v, list)| list.push(v.clone()));
        }

        let trial_ids: Vec<u32> = self.trials.iter().map(|t| t.id).collect();
        let mut series = vec![Series::new("trial_id", trial_ids)];

        let mut var_series: Vec<Series> = var_values
            .into_iter()
            .zip(vars)
            .map(|(list, name)| Series::new(&name, list))
            .collect();

        series.append(&mut var_series);

        let df = DataFrame::new(series);

        df
    }
}

pub fn decimal_to_f64(input: Vec<Decimal>) -> Vec<f64> {
    input
        .into_iter()
        .map(|v| v.to_f64().unwrap_or_default())
        .collect()
}

#[cfg(feature = "py-ext")]
pub fn convert_decimal(input: Decimal) -> AnyValue<'static> {
    AnyValue::Decimal(input.0.mantissa(), input.0.scale() as usize)
}

#[cfg(not(feature = "py-ext"))]
pub fn convert_decimal(input: Decimal) -> AnyValue<'static> {
    AnyValue::Decimal(input.mantissa(), input.scale() as usize)
}

pub fn decimal_to_arrow_decimal(input: Vec<Decimal>) -> Vec<AnyValue<'static>> {
    input.into_iter().map(convert_decimal).collect()
}

pub fn maybe_decimal_to_arrow_decimal(input: Vec<Option<Decimal>>) -> Vec<AnyValue<'static>> {
    input
        .into_iter()
        .map(|v| v.map(convert_decimal).unwrap_or(AnyValue::Null))
        .collect()
}

pub fn maybe_decimal_to_f64(input: Vec<Option<Decimal>>) -> Vec<Option<f64>> {
    input
        .into_iter()
        .map(|v| v.map(|v| v.to_f64().unwrap_or_default()))
        .collect()
}

impl Trial {
    pub fn samples(&self) -> PolarsResult<DataFrame> {
        let mut ls_time = Vec::new();
        let mut ls_left_pos_x = Vec::new();
        let mut ls_left_pos_y = Vec::new();
        let mut ls_left_area = Vec::new();
        let mut ls_right_pos_x = Vec::new();
        let mut ls_right_pos_y = Vec::new();
        let mut ls_right_area = Vec::new();
        let mut ls_res_x = Vec::new();
        let mut ls_res_y = Vec::new();

        for s in &self.samples {
            ls_time.push(s.time);
            ls_left_pos_x.push(s.left.as_ref().map(|e| e.position[0]));
            ls_left_pos_y.push(s.left.as_ref().map(|e| e.position[1]));
            ls_left_area.push(s.left.as_ref().map(|e| e.area));
            ls_right_pos_x.push(s.right.as_ref().map(|e| e.position[0]));
            ls_right_pos_y.push(s.right.as_ref().map(|e| e.position[1]));
            ls_right_area.push(s.right.as_ref().map(|e| e.area));
            ls_res_x.push(s.resolution.as_ref().map(|r| r[0]));
            ls_res_y.push(s.resolution.as_ref().map(|r| r[1]));
        }

        df! [
            "time" => decimal_to_arrow_decimal(ls_time),
            "left_pos_x" => maybe_decimal_to_arrow_decimal(ls_left_pos_x),
            "left_pos_y" => maybe_decimal_to_arrow_decimal(ls_left_pos_y),
            "right_pos_x" => maybe_decimal_to_arrow_decimal(ls_right_pos_x),
            "right_pos_y" => maybe_decimal_to_arrow_decimal(ls_right_pos_y),
            "left_area" => maybe_decimal_to_arrow_decimal(ls_left_area),
            "right_area" => maybe_decimal_to_arrow_decimal(ls_right_area),
            "res_x" => maybe_decimal_to_arrow_decimal(ls_res_x),
            "res_y" => maybe_decimal_to_arrow_decimal(ls_res_y),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_decimal_conversion() {
        let d = Decimal::from_str("12.23").unwrap();
        let c = convert_decimal(d);
        println!("{}", c);

        match c {
            AnyValue::Decimal(base, scale) => {
                let cons = rust_decimal::Decimal::new(base as i64, scale as u32);
                assert_eq!(cons, d);
            }
            _ => panic!("Invalid value returned"),
        }
    }
}
