use crate::asc::{Element, MsgType, PreambleMsg, RawSampleMsg, TrialData};
use crate::common::Eye;
use crate::{Decimal, NaiveDateTime};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "py-ext")]
use pyo3::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct Experiment {
    /// Additional metadata
    pub meta: MetaData,
    /// Defined labels for trial variables
    pub variable_labels: Vec<String>,
    pub trials: Vec<Trial>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct MetaData {
    pub recording_datetime: NaiveDateTime,
    pub preamble_lines: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct TimeRecord {
    pub start: Decimal,
    pub end: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct Trial {
    pub id: u32,
    pub time_record: TimeRecord,
    pub samples: Vec<Sample>,
    pub raw_samples: Vec<RawSample>,
    pub events: Vec<EventRecord>,
    pub camera_frames: Vec<CameraFrame>,
    pub variables: Vec<String>,
    pub targets: HashMap<String, Vec<TargetInfo>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct TargetInfo {
    time: Decimal,
    position: [u32; 2],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct EventRecord {
    time_record: TimeRecord,
    eye: Eye,
    resolution: Option<Vector>,
    info: EventInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventInfo {
    Fixation {
        average_position: Position,
        average_pupil_area: Decimal,
    },
    Saccade {
        start_position: Option<Position>,
        end_position: Option<Position>,
        movement_angle: Option<Decimal>,
        peak_velocity: Decimal,
    },
    Blink,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct Sample {
    pub time: Decimal,
    pub left: Option<EyeSampleData>,
    pub right: Option<EyeSampleData>,
    pub resolution: Option<Vector>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct CameraFrame {
    pub name: String,
    pub idx: u32,
    pub cam_time: u64,
    pub sys_time: u64,
    pub process_time: Decimal,
    pub eyelink_time: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct RawSample {
    pub time: Decimal,
    pub left: RawEyeSampleData,
    pub right: RawEyeSampleData,
}

pub type Position = [Decimal; 2];
pub type Vector = [Decimal; 2];

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct EyeSampleData {
    pub position: Position,
    pub area: Decimal,
    pub velocity: Option<Vector>,
    pub cr: CRStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub struct RawEyeSampleData {
    pub pupil_position: Position,
    pub pupil_area: Decimal,
    pub pupil_size: Vector,
    pub cr_position: Position,
    pub cr_area: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "py-ext", pyclass(get_all))]
pub enum CRStatus {
    Missing,
    Recovering,
    Found,
}

impl TimeRecord {
    pub fn new_checked(start: Decimal, end: Decimal, duration: Decimal) -> anyhow::Result<Self> {
        if end - start == duration {
            Ok(Self { start, end })
        } else {
            Err(anyhow!(
                "Duration ({}) does not match start and end time points {}, {}",
                duration,
                start,
                end
            ))
        }
    }
}

fn optional_pos(x: Option<Decimal>, y: Option<Decimal>) -> Option<Position> {
    x.and_then(|x| y.map(|y| [x, y]))
}

impl EventRecord {
    fn from_event_info(
        eye: Eye,
        start_time: Decimal,
        end_time: Decimal,
        duration: Decimal,
        resolution: Option<Vector>,
        info: EventInfo,
    ) -> Self {
        EventRecord {
            eye,
            time_record: TimeRecord::new_checked(start_time, end_time, duration).unwrap(),
            resolution,
            info,
        }
    }

    fn from_saccade_end(
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
    ) -> Self {
        let info = EventInfo::Saccade {
            start_position: optional_pos(start_pos_x, start_pos_y),
            end_position: optional_pos(end_pos_x, end_pos_y),
            movement_angle,
            peak_velocity,
        };
        Self::from_event_info(
            eye,
            start_time,
            end_time,
            duration,
            Some([res_x, res_y]),
            info,
        )
    }

    fn from_fixation_end(
        eye: Eye,
        start_time: Decimal,
        end_time: Decimal,
        duration: Decimal,
        average_pos_x: Decimal,
        average_pos_y: Decimal,
        average_pupil_size: Decimal,
        res_x: Decimal,
        res_y: Decimal,
    ) -> Self {
        let info = EventInfo::Fixation {
            average_position: [average_pos_x, average_pos_y],
            average_pupil_area: average_pupil_size,
        };
        Self::from_event_info(
            eye,
            start_time,
            end_time,
            duration,
            Some([res_x, res_y]),
            info,
        )
    }

    fn from_blink_end(eye: Eye, start_time: Decimal, end_time: Decimal, duration: Decimal) -> Self {
        Self::from_event_info(eye, start_time, end_time, duration, None, EventInfo::Blink)
    }
}

impl Trial {
    pub fn from_trial_start(id: u32, start_time: Decimal) -> Self {
        Trial {
            id,
            time_record: TimeRecord {
                start: start_time,
                end: Decimal::default(),
            },
            samples: Vec::new(),
            camera_frames: Vec::new(),
            raw_samples: Vec::new(),
            events: Vec::new(),
            variables: Vec::new(),
            targets: HashMap::new(),
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

impl RawEyeSampleData {
    pub fn from_asc(data: RawSampleMsg) -> RawEyeSampleData {
        RawEyeSampleData {
            pupil_position: [data.pupil_pos_x, data.pupil_pos_y],
            pupil_area: data.pupil_area,
            pupil_size: [data.pupil_size_x, data.pupil_size_y],
            cr_position: [data.cr_pos_x, data.cr_pos_y],
            cr_area: data.cr_area,
        }
    }
}

impl RawSample {
    pub fn from_asc(time: Decimal, left: RawSampleMsg, right: RawSampleMsg) -> RawSample {
        RawSample {
            time,
            left: RawEyeSampleData::from_asc(left),
            right: RawEyeSampleData::from_asc(right),
        }
    }
}

impl CameraFrame {
    pub fn from_asc(
        name: String,
        idx: u32,
        cam_time: u64,
        sys_time: u64,
        process_time: Decimal,
        eyelink_time: Decimal,
    ) -> Self {
        CameraFrame {
            name,
            idx,
            cam_time,
            sys_time,
            process_time,
            eyelink_time,
        }
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
        _unknown: Option<Decimal>,
        _interpolated: bool,
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

fn push_sample(sample: Sample, trials: &mut Vec<Trial>) {
    let last = trials.last_mut().expect("No trial");
    last.samples.push(sample);
}

fn push_event(event: EventRecord, trials: &mut Vec<Trial>) {
    let last = trials.last_mut().expect("No trial");
    last.events.push(event);
}

impl From<Vec<Element>> for Experiment {
    fn from(value: Vec<Element>) -> Self {
        let mut trials: Vec<Trial> = Vec::new();
        let mut variable_labels = None;
        let mut meta = MetaData::default();

        for el in value {
            match el {
                Element::Msg(time, msg_type) => match msg_type {
                    MsgType::CameraFrame {
                        name,
                        frame_idx,
                        cam_time,
                        sys_time,
                        process_time,
                        eyelink_time,
                    } => trials.last_mut().expect("No trial").camera_frames.push(
                        CameraFrame::from_asc(
                            name,
                            frame_idx,
                            cam_time,
                            sys_time,
                            process_time,
                            eyelink_time,
                        ),
                    ),
                    MsgType::RawData { time, left, right } => trials
                        .last_mut()
                        .expect("Raw sample outside trial")
                        .raw_samples
                        .push(RawSample::from_asc(time, left, right)),
                    MsgType::TrialId(id) => trials.push(Trial::from_trial_start(id, time)),
                    MsgType::TrialResult(_) => {
                        trials
                            .last_mut()
                            .expect("Invalid end of trial")
                            .time_record
                            .end = time
                    }
                    MsgType::TrialVarLabels(labels) => {
                        variable_labels = Some(labels);
                    }
                    MsgType::TrialData(data) => match data {
                        TrialData::VarValues(elems) => {
                            trials
                                .last_mut()
                                .expect("Trial variable data reported outside trial")
                                .variables = elems
                        }
                        TrialData::Targets(targets) => {
                            let trial = trials
                                .last_mut()
                                .expect("Target position reported outside trial");
                            for target in targets {
                                let info = TargetInfo {
                                    time,
                                    position: target.position,
                                };

                                match trial.targets.get_mut(&target.name) {
                                    Some(li) => li.push(info),
                                    None => {
                                        trial.targets.insert(target.name, vec![info]);
                                    }
                                };
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Element::Preamble(p) => match p {
                    PreambleMsg::DateTime(d) => meta.recording_datetime = d,
                    PreambleMsg::Other(s) => meta.preamble_lines.push(s),
                    PreambleMsg::Empty => {}
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
                    push_sample(s, &mut trials);
                }
                Element::SaccadeEnd {
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
                } => {
                    let rec = EventRecord::from_saccade_end(
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
                    );
                    push_event(rec, &mut trials);
                }
                Element::FixationEnd {
                    eye,
                    start_time,
                    end_time,
                    duration,
                    average_pos_x,
                    average_pos_y,
                    average_pupil_size,
                    res_x,
                    res_y,
                } => {
                    let rec = EventRecord::from_fixation_end(
                        eye,
                        start_time,
                        end_time,
                        duration,
                        average_pos_x,
                        average_pos_y,
                        average_pupil_size,
                        res_x,
                        res_y,
                    );
                    push_event(rec, &mut trials);
                }
                Element::BlinkEnd {
                    eye,
                    start_time,
                    end_time,
                    duration,
                } => {
                    let rec = EventRecord::from_blink_end(eye, start_time, end_time, duration);
                    push_event(rec, &mut trials);
                }
                _ => {}
            }
        }

        Experiment {
            trials,
            variable_labels: variable_labels.unwrap_or_default(),
            meta,
        }
    }
}
