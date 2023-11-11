/* NOTES
 * Generally trying to produce time series data of target cadence range
 * and target power.  Using 1 second intervals since that seems to
 * match the update rate of the BlueTooth Cycling Power Service.
 * There are a lot of other fields/properties in some of the workout
 * types, but I'm not going to mess with them unless it is critical
 * */

use super::{
    zwo_parse::{ExerciseTag, TimeSeriesError, Warmup},
    Workout,
};

/// struct for holding workout time series data
/// either workout section or full workout
pub struct WorkoutTimeSeries {
    pub time: Vec<usize>,
    pub cadence: Vec<i32>,
    pub power: Vec<f32>,
}

/// takes vec<WorkoutTag> and produces time series for the full workout file
pub fn create_timeseries(workout: Workout) -> Result<WorkoutTimeSeries, TimeSeriesError> {
    let mut final_duration: Vec<usize> = Vec::new();
    let mut final_cadence: Vec<i32> = Vec::new();
    let mut final_power: Vec<f32> = Vec::new();
    let tags = workout.exercise;
    for tag in tags.iter() {
        match tag {
            ExerciseTag::Warmup(warmup_struct) => {
                let mut intermediate = warmup_struct.to_time_series()?;
                final_duration.append(&mut intermediate.time);
                final_cadence.append(&mut intermediate.cadence);
                final_power.append(&mut intermediate.power);
            }
            _ => {}
        }
    }

    let final_series = WorkoutTimeSeries {
        time: final_duration,
        cadence: final_cadence,
        power: final_power,
    };
    return Ok(final_series);
}
