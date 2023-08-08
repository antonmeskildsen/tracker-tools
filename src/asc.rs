use anyhow::{anyhow, Context};

use crate::common::Eye;
use crate::{Decimal, NaiveDateTime};

use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Element {
    Preamble(PreambleMsg),
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
        start_pos_x: Option<Decimal>,
        start_pos_y: Option<Decimal>,
        end_pos_x: Option<Decimal>,
        end_pos_y: Option<Decimal>,
        movement_angle: Option<Decimal>,
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

#[derive(Debug, Clone)]
pub struct DataOptions {
    #[allow(unused)]
    res: bool,
    #[allow(unused)]
    rate: Decimal,
    #[allow(unused)]
    tracking: TrackingMode,
    #[allow(unused)]
    filter: FilterType,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Gaze,
    Href,
    Pupil,
}

#[derive(Debug, Clone)]
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
    TrialData(TrialData),
    CameraFrame {
        name: String,
        version: CameraFrameVersion,
        frame_idx: u32,
        cam_time: u64,
        sys_time: u64,
        process_time: Decimal,
        eyelink_time: Option<Decimal>,
    },
    Other(String),
    RawData {
        time: Decimal,
        left: RawSampleMsg,
        right: RawSampleMsg,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraFrameVersion {
    V1, // Used for unspecified version
    V2, // The first version where the number is specified in the .asc file
}

#[derive(Debug, Clone)]
pub struct RawSampleMsg {
    pub pupil_pos_x: Decimal,
    pub pupil_pos_y: Decimal,
    pub pupil_area: Decimal,
    pub pupil_size_x: Decimal,
    pub pupil_size_y: Decimal,
    pub cr_pos_x: Decimal,
    pub cr_pos_y: Decimal,
    pub cr_area: Decimal,
}

#[derive(Debug, Clone)]
pub enum PreambleMsg {
    DateTime(NaiveDateTime),
    Other(String),
    Empty,
}

#[derive(Debug, Clone)]
pub enum TrialData {
    VarValues(Vec<String>),
    Targets(Vec<Target>),
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub position: [i32; 2],
    pub visible: bool,
    pub interpolate: bool,
}

#[derive(Debug, Clone)]
pub enum TrackingAlgorithm {
    Ellipse,
    Centroid,
}

#[derive(Debug, Clone)]
pub struct ThresholdSpec {
    #[allow(unused)]
    pupil: u32,
    #[allow(unused)]
    cr: u32,
}

#[derive(Debug, Clone)]
pub enum TrackingMode {
    Pupil,
    CR,
}

#[derive(Debug, Clone)]
pub enum FilterType {
    Off,
    Standard,
    Extra,
}

#[derive(Debug, Clone)]
pub enum EyeSpecification {
    L,
    R,
    LR,
}

#[derive(Debug, Clone)]
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

pub fn from_decimal(s: &str) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str(s).or_else(|_| Decimal::from_scientific(s))
}

pub fn maybe_decimal(s: &str) -> anyhow::Result<Option<Decimal>> {
    (s != ".")
        .then(|| from_decimal(s))
        .map_or(Ok(None), |v| v.map(Some))
        .context("when converting decimal from string")
}

impl FromStr for Element {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            Ok(Element::Blank)
        } else {
            match parts[0] {
                "**" => Ok(Element::Preamble(PreambleMsg::from_str(
                    &s[min(parts[0].len() + 1, s.len())..],
                )?)),
                "MSG" => Ok(Element::Msg(
                    from_decimal(parts[1])?,
                    MsgType::from_str(&s[parts[0].len() + parts[1].len() + 1..])?,
                )),
                "#" | ";" | "//" => Ok(Element::Comment(s[parts[0].len() + 1..].to_string())),
                "INPUT" => Ok(Element::Input {
                    time: from_decimal(parts[1])?,
                    value: u32::from_str(parts[2])?,
                }),
                "SSACC" => {
                    let eye = Eye::from_str(parts[1])?;
                    let time = from_decimal(parts[2])?;
                    Ok(Element::SaccadeStart { eye, time })
                }
                "SFIX" => {
                    let eye = Eye::from_str(parts[1])?;
                    let time = from_decimal(parts[2])?;
                    Ok(Element::FixationStart { eye, time })
                }
                "SBLINK" => {
                    let eye = Eye::from_str(parts[1])?;
                    let time = from_decimal(parts[2])?;
                    Ok(Element::BlinkStart { eye, time })
                }
                "ESACC" => {
                    let eye = Eye::from_str(parts[1])?;
                    let start_time = from_decimal(parts[2])?;
                    let end_time = from_decimal(parts[3])?;
                    let duration = from_decimal(parts[4])?;
                    let start_pos_x = maybe_decimal(parts[5])?;
                    let start_pos_y = maybe_decimal(parts[6])?;
                    let end_pos_x = maybe_decimal(parts[7])?;
                    let end_pos_y = maybe_decimal(parts[8])?;
                    let movement_angle = maybe_decimal(parts[9])?;
                    let peak_velocity = from_decimal(parts[10])?;
                    let res_x = from_decimal(parts[11])?;
                    let res_y = from_decimal(parts[12])?;
                    Ok(Element::SaccadeEnd {
                        eye,
                        start_time,
                        end_time,
                        duration,
                        start_pos_x,
                        start_pos_y,
                        end_pos_x,
                        end_pos_y,
                        movement_angle,
                        peak_velocity,
                        res_x,
                        res_y,
                    })
                }
                "EFIX" => {
                    let eye = Eye::from_str(parts[1])?;
                    let start_time = from_decimal(parts[2])?;
                    let end_time = from_decimal(parts[3])?;
                    let duration = from_decimal(parts[4])?;
                    let average_pos_x = from_decimal(parts[5])?;
                    let average_pos_y = from_decimal(parts[6])?;
                    let average_pupil_size = from_decimal(parts[7])?;
                    let res_x = from_decimal(parts[8])?;
                    let res_y = from_decimal(parts[9])?;
                    Ok(Element::FixationEnd {
                        eye,
                        start_time,
                        end_time,
                        duration,
                        average_pos_x,
                        average_pos_y,
                        average_pupil_size,
                        res_x,
                        res_y,
                    })
                }
                "EBLINK" => {
                    let eye = Eye::from_str(parts[1])?;
                    let start_time = from_decimal(parts[2])?;
                    let end_time = from_decimal(parts[3])?;
                    let duration = from_decimal(parts[4])?;
                    Ok(Element::BlinkEnd {
                        eye,
                        start_time,
                        end_time,
                        duration,
                    })
                }
                _ => match from_decimal(parts[0]) {
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
                let sampling_rate = from_decimal(parts[2])?;
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
                let left = from_decimal(parts[1])?;
                let top = from_decimal(parts[2])?;
                let right = from_decimal(parts[3])?;
                let bottom = from_decimal(parts[4])?;
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
                from_decimal(parts[2])?,
            )),
            "CAMERA_LENS_FOCAL_LENGTH" => {
                Ok(MsgType::CameraLensFocalLength(from_decimal(parts[1])?))
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
            "!V" => Ok(MsgType::TrialData(TrialData::from_str(
                &s[parts[0].len() + 1..],
            )?)),
            "L" => {
                let time = from_decimal(parts[1])?;
                let left = RawSampleMsg::from_str_slice(&parts[2..10])?;
                let right = RawSampleMsg::from_str_slice(&parts[11..])?;
                Ok(MsgType::RawData { time, left, right })
            }
            "CAM_FRAME" => {
                let mut p_iter = parts[1..].iter();
                let first = p_iter.next().unwrap().to_string();
                let (version, name) = if first == "V2" {
                    (CameraFrameVersion::V2, p_iter.next().unwrap().to_string())
                } else {
                    (CameraFrameVersion::V1, first)
                };
                let frame_idx = u32::from_str(p_iter.next().unwrap())?;
                let cam_time = u64::from_str(p_iter.next().unwrap())?;
                let sys_time = u64::from_str(p_iter.next().unwrap())?;
                let process_time = from_decimal(p_iter.next().unwrap())?;
                let eyelink_time = p_iter.next().map(|v| from_decimal(v)).transpose()?;

                Ok(MsgType::CameraFrame {
                    name,
                    version,
                    frame_idx,
                    cam_time,
                    sys_time,
                    process_time,
                    eyelink_time,
                })
            }
            _ => Ok(MsgType::Other(s.to_string())),
        }
    }
}

impl RawSampleMsg {
    fn from_str_slice(parts: &[&str]) -> Result<Self, rust_decimal::Error> {
        Ok(RawSampleMsg {
            pupil_pos_x: from_decimal(parts[0])?,
            pupil_pos_y: from_decimal(parts[1])?,
            pupil_area: from_decimal(parts[2])?,
            pupil_size_x: from_decimal(parts[3])?,
            pupil_size_y: from_decimal(parts[4])?,
            cr_pos_x: from_decimal(parts[5])?,
            cr_pos_y: from_decimal(parts[6])?,
            cr_area: from_decimal(parts[7])?,
        })
    }
}

impl FromStr for PreambleMsg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(tp) = s.split_whitespace().next() {
            match tp {
                "DATE:" => Ok(PreambleMsg::DateTime(NaiveDateTime::parse_from_str(
                    &s[tp.len() + 1..],
                    "%a %b %d %H:%M:%S %Y",
                )?)),
                _ => Ok(PreambleMsg::Other(s.to_string())),
            }
        } else {
            Ok(PreambleMsg::Empty)
        }
    }
}

impl Target {
    fn from_str_slice(s: &[&str]) -> anyhow::Result<Self> {
        let name = s[0].to_string();
        let position_x_str = s[1];
        let position_x = i32::from_str(&position_x_str[1..position_x_str.len() - 1])?;
        let position_y_str = s[2];
        let position_y = i32::from_str(&position_y_str[..position_y_str.len() - 1])?;
        let visible = i32::from_str(s[3])? == 1;
        let interpolate = i32::from_str(s[4])? == 1;
        Ok(Target {
            name,
            position: [position_x, position_y],
            visible,
            interpolate,
        })
    }
}

impl FromStr for TrialData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts[0] {
            "TRIAL_VAR_DATA" => {
                let elems: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                Ok(TrialData::VarValues(elems))
            }
            "TARGET_POS" => {
                let mut targets = Vec::new();
                if parts.len() >= 6 {
                    targets.push(Target::from_str_slice(&parts[1..6])?)
                }
                if parts.len() == 11 {
                    targets.push(Target::from_str_slice(&parts[6..12])?)
                }

                Ok(TrialData::Targets(targets))
            }
            _ => Ok(TrialData::Other(s.to_string())),
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
