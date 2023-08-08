use ascc::generic::Sample;
use ascc::Decimal;
use egui::plot::{Line, PlotPoint, PlotPoints};
use std::fmt::Debug;

pub fn create_line<T, U1, U2, F1, F2>(collection: &[T], mut get_x: F1, mut get_y: F2) -> Line
where
    F1: FnMut(&T) -> Option<U1>,
    F2: FnMut(&T) -> Option<U2>,
    U1: TryInto<f64>,
    U2: TryInto<f64>,
    <U1 as TryInto<f64>>::Error: Debug,
    <U2 as TryInto<f64>>::Error: Debug,
{
    let points: PlotPoints = collection
        .iter()
        .filter_map(|s| Some([get_x(s)?.try_into().unwrap(), get_y(s)?.try_into().unwrap()]))
        .collect();
    Line::new(points)
}
