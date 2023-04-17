use crate::asc::Element::Other;
use anyhow::anyhow;

use rust_decimal::Decimal;
use std::cmp::min;
use std::str::FromStr;

// pub struct BinocularSample {
//     time: Decimal,
//     left: Option<EyeSample>,
//     right: Option<EyeSample>,
//     interpolated: bool,
//     left_cr_status: CRStatus,
//     right_cr_status: CRStatus,
// }
//
// pub enum CRStatus {
//     Found,
//     Recovering,
//     Missing,
// }
//
// pub struct EyeSample {
//     pos: [Decimal; 2],
//     pupil_area: Decimal,
//     velocity: Option<[Decimal; 2]>,
// }
//
// pub struct Message {
//     time: Decimal,
//     msg: String,
// }
//
// pub struct Trial {
//     id: u32,
//     samples: Vec<BinocularSample>
// }

pub enum Element {
    Preamble(String),
    Msg(Decimal, MsgType),
    Comment(String),
    Other(String),
    Input {
        time: Decimal,
        value: u32,
    },
    Sample {
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
    },
    StartBlock {
        time: Decimal,
        eye_left: bool,
        eye_right: bool,
        samples: bool,
        events: bool,
    },
    EndBlock {
        time: Decimal,
        samples: bool,
        events: bool,
        resolution: Option<[Decimal; 2]>,
    },
    FixationStart {
        eye: Eye,
        time: Decimal,
    },
    FixationEnd {
        eye: Eye,
        start_time: Decimal,
        end_time: Decimal,
        duration: Decimal,
        average_pos_x: Decimal,
        average_pos_y: Decimal,
        average_pupil_size: Decimal,
        res_x: Decimal,
        res_y: Decimal,
    },
    SaccadeStart {
        eye: Eye,
        time: Decimal,
    },
    SaccadeEnd {
        eye: Eye,
        start_time: Decimal,
        end_time: Decimal,
        duration: Decimal,
        start_pos_x: Decimal,
        start_pos_y: Decimal,
        end_pos_x: Decimal,
        end_pos_y: Decimal,
        movement_angle: Decimal,
        peak_velocity: Decimal,
        res_x: Decimal,
        res_y: Decimal,
    },
    BlinkStart {
        eye: Eye,
        time: Decimal,
    },
    BlinkEnd {
        eye: Eye,
        start_time: Decimal,
        end_time: Decimal,
        duration: Decimal,
    },
    PrescalerPosition(Decimal),
    PrescalerVelocity(Decimal),
    EventSpec {
        data_type: DataType,
        left_eye: bool,
        right_eye: bool,
        options: DataOptions,
    },
    SampleSpec {
        data_type: DataType,
        left_eye: bool,
        right_eye: bool,
        velocity: bool,
        options: DataOptions,
    },
    Blank,
}

pub struct DataOptions {
    res: bool,
    rate: Decimal,
    tracking: TrackingMode,
    filter: FilterType,
}

pub enum DataType {
    Gaze,
    Href,
    Pupil,
}

pub enum Eye {
    L,
    R,
}

pub enum MsgType {
    TrialId(u32),
    TrialResult(u32),
    RecordingConfiguration {
        tracking_mode: TrackingMode,
        sampling_rate: Decimal,
        file_sample_filter: FilterType,
        link_sample_filter: FilterType,
        eyes: EyeSpecification,
    },
    MountConfiguration(MountConfiguration),
    GazeCoordinates {
        left: Decimal,
        top: Decimal,
        right: Decimal,
        bottom: Decimal,
    },
    Thresholds {
        left: ThresholdSpec,
        right: ThresholdSpec,
    },
    TrackingAlgorithm(TrackingAlgorithm),
    PcrParameter(u32, Decimal),
    CameraLensFocalLength(Decimal),
    WindowSizes(u32, u32, u32, u32),
    PupilDataType(String),
    TrialVarLabels(Vec<String>),
    TrialVarGrouping(Vec<String>),
    TrialVarData(Vec<(String, String)>),
    TargetPos(Vec<Target>),
    Other(String),
}

pub struct Target {
    name: String,
    position: [u32; 2],
    visible: bool,
    interpolate: bool,
}

pub enum TrackingAlgorithm {
    Ellipse,
    Centroid,
}

pub struct ThresholdSpec {
    pupil: u32,
    cr: u32,
}

pub enum TrackingMode {
    Pupil,
    CR,
}

pub enum FilterType {
    Off,
    Standard,
    Extra,
}

pub enum EyeSpecification {
    L,
    R,
    LR,
}

pub enum MountConfiguration {
    MTABLER,
    BTABLER,
    RTABLER,
    RBTABLER,
    AMTABLER,
    ARTABLER,
    BTOWER,
    TOWER,
    MPRIM,
    BPRIM,
    MLRR,
    BLRR,
}

pub fn maybe_decimal(s: &str) -> Result<Option<Decimal>, rust_decimal::Error> {
    (s != ".")
        .then(|| Decimal::from_str(s))
        .map_or(Ok(None), |v| v.map(Some))
}

impl FromStr for Element {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() == 0 {
            Ok(Element::Blank)
        } else {
            match parts[0] {
                "**" => Ok(Element::Preamble(
                    s[min(parts[0].len() + 1, s.len())..].to_string(),
                )),
                "MSG" => Ok(Element::Msg(
                    Decimal::from_str(parts[1])?,
                    MsgType::from_str(&s[parts[0].len() + parts[1].len() + 1..])?,
                )),
                "#" | ";" | "//" => Ok(Element::Comment(s[parts[0].len() + 1..].to_string())),
                "INPUT" => Ok(Element::Input {
                    time: Decimal::from_str(parts[1])?,
                    value: u32::from_str(parts[2])?,
                }),
                _ => match Decimal::from_str(parts[0]) {
                    Err(_) => Ok(Element::Other(s.to_string())),
                    Ok(time) => {
                        let left_pos_x = maybe_decimal(parts[1])?;
                        let left_pos_y = maybe_decimal(parts[2])?;
                        let left_area = maybe_decimal(parts[3])?;
                        let right_pos_x = maybe_decimal(parts[4])?;
                        let right_pos_y = maybe_decimal(parts[5])?;
                        let right_area = maybe_decimal(parts[6])?;
                        let left_velocity_x = maybe_decimal(parts[7])?;
                        let left_velocity_y = maybe_decimal(parts[8])?;
                        let right_velocity_x = maybe_decimal(parts[9])?;
                        let right_velocity_y = maybe_decimal(parts[10])?;
                        let res_x = maybe_decimal(parts[11])?;
                        let res_y = maybe_decimal(parts[12])?;
                        let unknown = maybe_decimal(parts[13])?;
                        let mut options = parts[14].chars();
                        let interpolated =
                            options.next().ok_or(anyhow!("Invalid options 0"))? == 'I';
                        let left_cr_missing =
                            options.next().ok_or(anyhow!("Invalid options 1"))? == 'C';
                        let left_cr_recovering =
                            options.next().ok_or(anyhow!("Invalid options 2"))? == 'R';
                        let right_cr_missing =
                            options.next().ok_or(anyhow!("Invalid options 3"))? == 'C';
                        let right_cr_recovering =
                            options.next().ok_or(anyhow!("Invalid options 4"))? == 'R';

                        Ok(Element::Sample {
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
                        })
                    }
                },
            }
        }
    }
}

impl FromStr for MsgType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts[0] {
            "TRIALID" => Ok(MsgType::TrialId(u32::from_str(parts[1])?)),
            "TRIAL_RESULT" => Ok(MsgType::TrialResult(u32::from_str(parts[1])?)),
            "RECCFG" => {
                let tracking_mode = TrackingMode::from_str(parts[1])?;
                let sampling_rate = Decimal::from_str(parts[2])?;
                let file_sample_filter = FilterType::from_str(parts[3])?;
                let link_sample_filter = FilterType::from_str(parts[4])?;
                let eyes = EyeSpecification::from_str(parts[5])?;
                Ok(MsgType::RecordingConfiguration {
                    tracking_mode,
                    sampling_rate,
                    file_sample_filter,
                    link_sample_filter,
                    eyes,
                })
            }
            "ELCLCFG" => Ok(MsgType::MountConfiguration(MountConfiguration::from_str(
                parts[1],
            )?)),
            "GAZE_COORDS" => {
                let left = Decimal::from_str(parts[1])?;
                let top = Decimal::from_str(parts[2])?;
                let right = Decimal::from_str(parts[3])?;
                let bottom = Decimal::from_str(parts[4])?;
                Ok(MsgType::GazeCoordinates {
                    left,
                    top,
                    right,
                    bottom,
                })
            }
            "THRESHOLDS" => {
                let left_pupil = u32::from_str(parts[2])?;
                let left_cr = u32::from_str(parts[3])?;
                let right_pupil = u32::from_str(parts[5])?;
                let right_cr = u32::from_str(parts[6])?;
                Ok(MsgType::Thresholds {
                    left: ThresholdSpec {
                        pupil: left_pupil,
                        cr: left_cr,
                    },
                    right: ThresholdSpec {
                        pupil: right_pupil,
                        cr: right_cr,
                    },
                })
            }
            "ELCL_PROC" => Ok(MsgType::TrackingAlgorithm(TrackingAlgorithm::from_str(
                parts[1],
            )?)),
            "ELCL_PCR_PARAM" => Ok(MsgType::PcrParameter(
                u32::from_str(parts[1])?,
                Decimal::from_str(parts[2])?,
            )),
            "CAMERA_LENS_FOCAL_LENGTH" => {
                Ok(MsgType::CameraLensFocalLength(Decimal::from_str(parts[1])?))
            }
            "ELCL_WINDOW_SIZES" => {
                let a = u32::from_str(parts[1])?;
                let b = u32::from_str(parts[2])?;
                let c = u32::from_str(parts[3])?;
                let d = u32::from_str(parts[4])?;
                Ok(MsgType::WindowSizes(a, b, c, d))
            }
            "PUPIL_DATA_TYPE" => Ok(MsgType::PupilDataType(parts[1].to_string())),
            "TRIAL_VAR_LABELS" => {
                let elems = parts[1..].iter().map(|s| s.to_string()).collect();
                Ok(MsgType::TrialVarLabels(elems))
            }
            _ => Ok(MsgType::Other(s.to_string())),
        }
    }
}

impl FromStr for TrackingAlgorithm {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ELLIPSE" => Ok(Self::Ellipse),
            "CENTROID" => Ok(Self::Centroid),
            _ => Err(anyhow!(format!("Invalid tracking algorithm: {s}"))),
        }
    }
}

impl FromStr for TrackingMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P" => Ok(TrackingMode::Pupil),
            "CR" => Ok(TrackingMode::CR),
            _ => Err(anyhow!(format!("Invalid tracking mode: {s}"))),
        }
    }
}

impl FromStr for FilterType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(FilterType::Off),
            "1" => Ok(FilterType::Standard),
            "2" => Ok(FilterType::Extra),
            _ => Err(anyhow!(format!("Invalid filter type: {s}"))),
        }
    }
}

impl FromStr for EyeSpecification {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(Self::L),
            "R" => Ok(Self::R),
            "LR" => Ok(Self::LR),
            _ => Err(anyhow!(format!("Invalid eye specification string: {s}"))),
        }
    }
}

impl FromStr for MountConfiguration {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MATBLER" => Ok(Self::MTABLER),
            "BTABLER" => Ok(Self::BTABLER),
            "RTABLER" => Ok(Self::RTABLER),
            "RBTABLER" => Ok(Self::RBTABLER),
            "AMTABLER" => Ok(Self::AMTABLER),
            "ARTABLER" => Ok(Self::ARTABLER),
            "BTOWER" => Ok(Self::BTOWER),
            "TOWER" => Ok(Self::TOWER),
            "MPRIM" => Ok(Self::MPRIM),
            "BPRIM" => Ok(Self::BPRIM),
            "MLRR" => Ok(Self::MLRR),
            "BLRR" => Ok(Self::BLRR),
            _ => Err(anyhow!(format!(
                "Invalid mounting configuration string: {s}"
            ))),
        }
    }
}
