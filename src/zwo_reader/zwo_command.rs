/* NOTES
 * Generally trying to produce time series data of target cadence range
 * and target power.  Using 1 second intervals since that seems to
 * match the update rate of the BlueTooth Cycling Power Service.
 * There are a lot of other fields/properties in some of the workout
 * types, but I'm not going to mess with them unless it is critical
 * */

/// struct for holding workout time series data
/// either workout section or full workout
pub struct WorkoutTimeSeries {
    time: Vec<usize>,
    cadence_low: Vec<i32>,
    cadence: Vec<i32>,
    cadence_high: Vec<i32>,
    power_low: Vec<f32>,
    power: Vec<f32>,
    power_high: Vec<f32>,
}
