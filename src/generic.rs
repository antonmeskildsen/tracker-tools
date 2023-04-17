use crate::asc::{Element, MsgType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GazeData {
    pub trials: Vec<Trial>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trial {
    pub id: u32,
    pub samples: Vec<Sample>,
    //events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sample {
    pub time: Decimal,
    pub left: Option<EyeSampleData>,
    pub right: Option<EyeSampleData>,
    pub resolution: Option<Vector>,
}

pub type Position = [Decimal; 2];
pub type Vector = [Decimal; 2];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EyeSampleData {
    pub position: Position,
    pub area: Decimal,
    pub velocity: Option<Vector>,
    pub cr: CRStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CRStatus {
    Missing,
    Recovering,
    Found,
}

impl Trial {
    pub fn new(id: u32) -> Self {
        Trial {
            id,
            samples: Vec::default(),
        }
    }
}

impl CRStatus {
    pub fn from_asc(cr_missing: bool, cr_recovering: bool) -> Self {
        if cr_missing {
            CRStatus::Missing
        } else if cr_recovering {
            CRStatus::Recovering
        } else {
            CRStatus::Found
        }
    }
}

impl EyeSampleData {
    pub fn from_asc(
        pos_x: Option<Decimal>,
        pos_y: Option<Decimal>,
        area: Option<Decimal>,
        velocity_x: Option<Decimal>,
        velocity_y: Option<Decimal>,
        cr_missing: bool,
        cr_recovering: bool,
    ) -> Option<EyeSampleData> {
        Some(EyeSampleData {
            position: [pos_x?, pos_y?],
            area: area?,
            velocity: velocity_x.and_then(|x| velocity_y.and_then(|y| Some([x, y]))),
            cr: CRStatus::from_asc(cr_missing, cr_recovering),
        })
    }
}

impl Sample {
    pub fn from_asc(
        time: Decimal,
        left_pos_x: Option<Decimal>,
        left_pos_y: Option<Decimal>,
        left_area: Option<Decimal>,
        right_pos_x: Option<Decimal>,
        right_pos_y: Option<Decimal>,
        right_area: Option<Decimal>,
        left_velocity_x: Option<Decimal>,
        left_velocity_y: Option<Decimal>,
        right_velocity_x: Option<Decimal>,
        right_velocity_y: Option<Decimal>,
        res_x: Option<Decimal>,
        res_y: Option<Decimal>,
        unknown: Option<Decimal>,
        interpolated: bool,
        left_cr_missing: bool,
        left_cr_recovering: bool,
        right_cr_missing: bool,
        right_cr_recovering: bool,
    ) -> Self {
        Sample {
            time,
            left: EyeSampleData::from_asc(
                left_pos_x,
                left_pos_y,
                left_area,
                left_velocity_x,
                left_velocity_y,
                left_cr_missing,
                left_cr_recovering,
            ),
            right: EyeSampleData::from_asc(
                right_pos_x,
                right_pos_y,
                right_area,
                right_velocity_x,
                right_velocity_y,
                right_cr_missing,
                right_cr_recovering,
            ),
            resolution: res_x.and_then(|x| res_y.and_then(|y| Some([x, y]))),
        }
    }
}

impl From<Vec<Element>> for GazeData {
    fn from(value: Vec<Element>) -> Self {
        let mut trials = Vec::new();

        for el in value {
            match el {
                Element::Msg(time, msg_type) => match msg_type {
                    MsgType::TrialId(id) => trials.push(Trial::new(id)),
                    _ => {}
                },
                Element::Sample {
                    time,
                    left_pos_x,
                    left_pos_y,
                    left_area,
                    right_pos_x,
                    right_pos_y,
                    right_area,
                    left_velocity_x,
                    left_velocity_y,
                    right_velocity_x,
                    right_velocity_y,
                    res_x,
                    res_y,
                    unknown,
                    interpolated,
                    left_cr_missing,
                    left_cr_recovering,
                    right_cr_missing,
                    right_cr_recovering,
                } => {
                    if let Some(t) = trials.last_mut() {
                        let s = Sample::from_asc(
                            time,
                            left_pos_x,
                            left_pos_y,
                            left_area,
                            right_pos_x,
                            right_pos_y,
                            right_area,
                            left_velocity_x,
                            left_velocity_y,
                            right_velocity_x,
                            right_velocity_y,
                            res_x,
                            res_y,
                            unknown,
                            interpolated,
                            left_cr_missing,
                            left_cr_recovering,
                            right_cr_missing,
                            right_cr_recovering,
                        );
                        t.samples.push(s)
                    }
                }
                _ => {}
            }
        }

        GazeData { trials }
    }
}
