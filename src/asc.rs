use rust_decimal::Decimal;

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

pub enum AscLine {
    Preamble(String),
    Msg(Decimal, MsgType),
    Comment(String),
    Sample {
        time: Decimal,
        left_pos_x: Decimal,
        left_pos_y: Decimal,
        left_area: Decimal,
        right_pos_x: Decimal,
        right_pos_y: Decimal,
        right_area: Decimal,
        left_velocity_x: Option<Decimal>,
        left_velocity_y: Option<Decimal>,
        right_velocity_x: Option<Decimal>,
        right_velocity_y: Option<Decimal>,
        res_x: Decimal,
        res_y: Decimal,
        unknown: Decimal,
        interpolated: bool,
        left_cr_missing: bool,
        left_cr_recovery: bool,
        right_cr_missing: bool,
        right_cr_recovery: bool,
    },
    StartBlock{
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
        time: Decimal
    },
    BlinkEnd {
        eye: Eye,
        start_time: Decimal,
        end_time: Decimal,
        duration: Decimal,
    }
}

pub enum Eye {
    L, R,
}

pub enum MsgType {
    TrialId(u32),
    TrialResult(u32),
    RecordingConfiguration{
        tracking_model: TrackingMode,
        sampling_rate: u32,
        sample_filter: FilterType,
        link_filter: FilterType,
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
    PupilCR,
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