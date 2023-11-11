use crate::zwo_reader;
use std::{fmt, fs};

use super::zwo_command::WorkoutTimeSeries;

/*===================================================================================
 * ENUMS
 * ================================================================================*/
#[derive(Debug)]
pub enum ExerciseTag {
    Warmup(Warmup),
    SteadyState(SteadyState),
    Cooldown(Cooldown),
    FreeRide(FreeRide),
    Freeride(Freeride), // yes this is a separate tag, zwo files are horrible
    IntervalsT(IntervalsT),
    MaxEffort(MaxEffort),
    Ramp(Ramp),
    RestDay, // IDK what this one does, there isn't any documentation
    SolidState(SolidState),
    Unknown, // TODO: evaluate if this is necessary
}

#[derive(Debug)]
pub enum TextTags {
    TextEvent(TextEvent),
    TextEvent2(Textevent),
    TextNotification(TextNotification),
    Unknown,
}

/*===================================================================================
 * STRUCTURES
 * ================================================================================*/
// TODO: make the error messages a bit more helpful (identify missing field)
pub struct TimeSeriesError;

impl fmt::Display for TimeSeriesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Missing essential field.") // user facing
    }
}

impl fmt::Debug for TimeSeriesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Missing essential field.") // programmer facing
    }
}

#[derive(Debug)]
pub struct Warmup {
    pub cadence: Option<i32>,
    pub cadence_high: Option<i32>,
    pub cadence_low: Option<i32>,
    pub cadence_resting: Option<i32>,
    pub duration: Option<i32>,
    pub pace: Option<i32>,
    pub power: Option<f32>,
    pub power_high: Option<f32>,
    pub power_low: Option<f32>,
    pub quantize: Option<i32>,
    pub replacement_prescription: Option<String>, // I don't think this one is necessary
    pub replacement_verb: Option<String>,         // also unnecessary
    pub text: Option<String>,
    pub units: Option<i32>, // no idea what this one does
    pub zone: Option<i32>,  // heart rate zone, maybe?
}

impl Warmup {
    fn to_time_series(&self) -> Result<WorkoutTimeSeries, TimeSeriesError> {
        // minimum required information - cadence, power target, duration
        let mut b_duration = false;
        let mut b_cadence = false;
        let mut b_power = false;
        let mut constant_cadence = true;
        let mut constant_power = true;
        if self.duration.is_some() {
            b_duration = true;
        }
        if self.cadence.is_some() || (self.cadence_low.is_some() && self.cadence_high.is_some()) {
            b_cadence = true;
            if self.cadence_low.is_some() && self.cadence_high.is_some() {
                constant_cadence = false;
            }
        }
        if self.power.is_some() || (self.power_low.is_some() && self.power_high.is_some()) {
            b_power = true;
            if self.power_low.is_some() && self.power_high.is_some() {
                constant_power = false;
            }
        }
        if b_power && b_cadence && b_duration {
            let duration = self.duration.unwrap();
            let duration_vec: Vec<usize> = (0..duration as usize).collect();
            let cadence: Vec<i32>;
            if constant_cadence {
                // create vector matching duration length, all one value
                cadence = vec![self.cadence.unwrap(); duration_vec.len()];
            } else {
                // use linear interpolation to create vector from high to low
            }
        } else {
            return Err(TimeSeriesError);
        }
    }
}

#[derive(Debug)]
pub struct SteadyState {
    pub cadence: Option<i32>,
    pub cadence_high: Option<i32>,
    pub cadence_low: Option<i32>,
    pub cadence_resting: Option<i32>,
    pub duration: Option<i32>,
    pub fail_threshold_duration: Option<f32>,
    pub forced_performance_test1: Option<i32>,
    pub forced_performance_test2: Option<i32>, // it can be camel case or snake case in zwo files ¯\_(ツ)_/¯
    pub never_fails: Option<bool>,
    pub off_power: Option<f32>,
    pub pace: Option<i32>,
    pub power: Option<f32>,
    pub power_high: Option<f32>,
    pub power_low: Option<f32>,
    pub ramp_test: Option<bool>,
    pub replacement_prescription: Option<String>, // I don't think this one is necessary
    pub replacement_verb: Option<String>,         // also unnecessary
    pub show_average: Option<bool>,
    pub target: Option<f32>,
    pub text: Option<String>,
    pub units: Option<i32>, // no idea what this one does
    pub zone: Option<i32>,  // heart rate zone, maybe?
}

#[derive(Debug)]
pub struct Cooldown {
    pub cadence: Option<i32>,
    pub cadence_high: Option<i32>,
    pub cadence_low: Option<i32>,
    pub cadence_resting: Option<i32>,
    pub duration: Option<i32>,
    pub end_at_road_time: Option<f32>,
    pub pace: Option<i32>,
    pub pace2: Option<i32>, // yeah there are two paces, lmao
    pub power: Option<f32>,
    pub power_high: Option<f32>,
    pub power_low: Option<f32>,
    pub replacement_prescription: Option<String>, // I don't think this one is necessary
    pub replacement_verb: Option<String>,         // also unnecessary
    pub units: Option<i32>,                       // no idea what this one does
    pub zone: Option<i32>,                        // heart rate zone, maybe?
}

#[derive(Debug)]
pub struct FreeRide {
    pub cadence: Option<i32>,
    pub cadence_high: Option<i32>,
    pub cadence_low: Option<i32>,
    pub duration: Option<i32>,
    pub fail_threshold_duration: Option<f32>,
    pub flat_road: Option<bool>,
    pub ftp_test: Option<bool>,
    pub power: Option<f32>,
    pub ramp_test: Option<bool>,
    pub show_average: Option<bool>,
}

#[derive(Debug)]
pub struct Freeride {
    pub duration: Option<i32>,
    pub flat_road: Option<bool>,
    pub ftp_test: Option<bool>,
}

#[derive(Debug)]
pub struct IntervalsT {
    pub cadence: Option<i32>,
    pub cadence_high: Option<i32>,
    pub cadence_low: Option<i32>,
    pub cadence_resting: Option<i32>,
    pub flat_road: Option<bool>,
    pub off_duration: Option<f32>,
    pub off_power: Option<f32>,
    pub on_duration: Option<f32>,
    pub on_power: Option<f32>,
    pub over_under: Option<bool>,
    pub pace: Option<i32>,
    pub power_off_high: Option<f32>,
    pub power_off_low: Option<f32>,
    pub power_off_zone: Option<i32>,
    pub power_on_high: Option<f32>,
    pub power_on_low: Option<f32>,
    pub power_on_zone: Option<i32>,
    pub repeat: Option<i32>,
    pub units: Option<i32>, // no idea what this one does
}

#[derive(Debug)]
pub struct MaxEffort {
    pub duration: Option<i32>,
}

#[derive(Debug)]
pub struct Ramp {
    pub cadence: Option<i32>,
    pub cadence_resting: Option<i32>,
    pub duration: Option<i32>,
    pub pace: Option<i32>,
    pub power: Option<f32>,
    pub power_high: Option<f32>,
    pub power_low: Option<f32>,
    pub show_average: Option<bool>,
}

#[derive(Debug)]
pub struct SolidState {
    pub duration: Option<i32>,
    pub power: Option<f32>,
}

#[derive(Debug)]
pub struct TextEvent {
    pub duration: Option<i32>,
    pub message: Option<String>,
    pub time_offset: Option<i32>,
    pub time_offset2: Option<i32>, // of course there are two, lmao
    pub level: Option<usize>,
    pub previous_element: Option<usize>,
}

// yeah, there are two different text events, and it's not just spelling, they have different fields for some reason
#[derive(Debug)]
pub struct Textevent {
    pub distoffset: Option<i32>, // distance offset
    pub duration: Option<i32>,
    pub message: Option<String>,
    pub text_scale: Option<i32>,
    pub time_offset: Option<i32>,
    pub level: Option<usize>,
    pub previous_element: Option<usize>,
}

#[derive(Debug)]
pub struct TextNotification {
    pub duration: Option<i32>,
    pub text: Option<String>,
    pub time_offset: Option<i32>,
    pub level: Option<usize>,
    pub previous_element: Option<usize>,
}

/*===================================================================================
 * FUNCTIONS
 * ================================================================================*/

// I could probably use an XML reader but I'm going to take the cheap way out
// some tags don't have a closing tag, which is annoying lmao

// this seems really stupid, lol
/// returns the names of Zwift file tags, couldn't figure out how to make them global
pub fn get_tag_names() -> (Vec<String>, Vec<String>, Vec<String>) {
    let category_names: Vec<String> = vec![
        "author".to_string(),
        "name".to_string(),
        "description".to_string(),
        "workout".to_string(), // TODO: currently this matches outer "workout_file" tag
        "sportType".to_string(),
        "tags".to_string(),
    ];

    let exercise_names: Vec<String> = vec![
        "Warmup".to_string(),
        "SteadyState".to_string(),
        "Cooldown".to_string(),
        "FreeRide".to_string(),
        "Freeride".to_string(), // yes, this is a different tag, zwo files are horrible
        "IntervalsT".to_string(),
        "MaxEffort".to_string(),
        "Ramp".to_string(),
        "RestDay".to_string(),
        "SolidState".to_string(),
    ];

    let text_names: Vec<String> = vec![
        "textevent".to_string(),
        "TextEvent".to_string(),
        "TextNotification".to_string(),
    ];

    return (category_names, exercise_names, text_names);
}

/// takes filename as a String and returns file contents as a String
pub fn file_to_text(filename: String) -> Result<String, std::io::Error> {
    let read_result = fs::read_to_string(filename);
    return read_result;
}

/// searchs for top level tags in a String of .zwo file contents
pub fn find_top_tags(file_contents: &String, tags: &Vec<String>) -> Vec<usize> {
    let mut tag_starts: Vec<usize> = Vec::new();
    // search for each tag in the file contents
    for tag in tags.iter() {
        let start_str = format!("<{}", tag);
        let start_pos: Vec<_> = file_contents
            .match_indices(&start_str)
            .map(|(i, _)| i)
            .collect();
        tag_starts.extend(start_pos);
    }
    return tag_starts;
}

/// helper function to get sections of a string
/// takes string, start position, and end position and returns substring from start:end
pub fn get_substr(full_str: &str, start: usize, end: usize) -> String {
    let mut substr = String::new();

    // check for valid indices
    if start > end {
        substr.push_str("");
    } else {
        for i in start..end {
            let index_result = full_str.chars().nth(i);
            match index_result {
                Some(c) => substr.push_str(&c.to_string()),
                None => {}
            }
        }
    }

    return substr;
}

/// converts tag string into the appropriate ExerciseTag Enum
pub fn exercise_str2enum(name_str: &str) -> ExerciseTag {
    let exercise_enum: ExerciseTag;
    match name_str {
        "Warmup" => {
            let new_struct = new_warmup_struct();
            exercise_enum = ExerciseTag::Warmup(new_struct);
        }
        "SteadyState" => {
            let new_struct = new_steady_state_struct();
            exercise_enum = ExerciseTag::SteadyState(new_struct);
        }
        "Cooldown" => {
            let new_struct = new_cooldown_struct();
            exercise_enum = ExerciseTag::Cooldown(new_struct);
        }
        "FreeRide" => {
            let new_struct = new_free_ride_struct();
            exercise_enum = ExerciseTag::FreeRide(new_struct);
        }
        "Freeride" => {
            let new_struct = new_free_ride2_struct();
            exercise_enum = ExerciseTag::Freeride(new_struct);
        }
        "IntervalsT" => {
            let new_struct = new_intervalst_struct();
            exercise_enum = ExerciseTag::IntervalsT(new_struct);
        }
        "MaxEffort" => {
            let new_struct = new_max_effort_struct();
            exercise_enum = ExerciseTag::MaxEffort(new_struct);
        }
        "Ramp" => {
            let new_struct = new_ramp_struct();
            exercise_enum = ExerciseTag::Ramp(new_struct);
        }
        "RestDay" => exercise_enum = ExerciseTag::RestDay,
        "SolidState" => {
            let new_struct = new_solid_state_struct();
            exercise_enum = ExerciseTag::SolidState(new_struct);
        }
        _ => exercise_enum = ExerciseTag::Unknown,
    }
    return exercise_enum;
}

/// converts text tag string into the appropriate TextTags Enum
pub fn text_str2enum(name_str: &str) -> TextTags {
    let text_enum: TextTags;
    match name_str {
        "TextEvent" => {
            let new_struct = new_text_event_struct();
            text_enum = TextTags::TextEvent(new_struct);
        }
        "textevent" => {
            let new_struct = new_text_event2_struct();
            text_enum = TextTags::TextEvent2(new_struct);
        }
        "TextNotification" => {
            let new_struct = new_text_notification_struct();
            text_enum = TextTags::TextNotification(new_struct);
        }
        _ => text_enum = TextTags::Unknown,
    }
    return text_enum;
}

/// gets list of tag properties for 'Warmup' tag
pub fn get_properties_warmup(chunk: &str) -> Warmup {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Cadence".to_string(),
        "CadenceHigh".to_string(),
        "CadenceLow".to_string(),
        "CadenceResting".to_string(),
        "Duration".to_string(),
        "pace".to_string(),
        "Power".to_string(),
        "PowerHigh".to_string(),
        "PowerLow".to_string(),
        "Quantize".to_string(),
        "replacement_prescription".to_string(),
        "replacement_verb".to_string(),
        "Text".to_string(),
        "units".to_string(),
        "Zone".to_string(),
    ];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_warmup_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "Cadence" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence = prop_value;
                }
                "CadenceHigh" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_high = prop_value;
                }
                "CadenceLow" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_low = prop_value;
                }
                "CadenceResting" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_resting = prop_value;
                }
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "pace" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.pace = prop_value;
                }
                "Power" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power = prop_value;
                }
                "PowerHigh" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_high = prop_value;
                }
                "PowerLow" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_low = prop_value;
                }
                "Quantize" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.quantize = prop_value;
                }
                "replacement_prescription" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.replacement_prescription = prop_value;
                }
                "replacement_verb" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.replacement_verb = prop_value;
                }
                "Text" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.text = prop_value;
                }
                "units" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.units = prop_value;
                }
                "Zone" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.zone = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'SteadyState' tag
pub fn get_properties_steady_state(chunk: &str) -> SteadyState {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Cadence".to_string(),
        "CadenceHigh".to_string(),
        "CadenceLow".to_string(),
        "CadenceResting".to_string(),
        "Duration".to_string(),
        "FailThresholdDuration".to_string(),
        "Forced_Performance_Test".to_string(),
        "forced_performance_test".to_string(),
        "NeverFails".to_string(),
        "OffPower".to_string(),
        "pace".to_string(),
        "Power".to_string(),
        "PowerHigh".to_string(),
        "PowerLow".to_string(),
        "ramptest".to_string(),
        "replacement_prescription".to_string(),
        "replacement_verb".to_string(),
        "show_avg".to_string(),
        "Target".to_string(),
        "Text".to_string(),
        "units".to_string(),
        "Zone".to_string(),
    ];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_steady_state_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "Cadence" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence = prop_value;
                }
                "CadenceHigh" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_high = prop_value;
                }
                "CadenceLow" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_low = prop_value;
                }
                "CadenceResting" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_resting = prop_value;
                }
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "FailThresholdDuration" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.fail_threshold_duration = prop_value;
                }
                "Forced_Performance_Test" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.forced_performance_test1 = prop_value;
                }
                "forced_performance_test" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.forced_performance_test2 = prop_value;
                }
                "NeverFails" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.never_fails = prop_value;
                }
                "OffPower" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.off_power = prop_value;
                }
                "pace" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.pace = prop_value;
                }
                "Power" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power = prop_value;
                }
                "PowerHigh" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_high = prop_value;
                }
                "PowerLow" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_low = prop_value;
                }
                "ramptest" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.ramp_test = prop_value;
                }
                "replacement_prescription" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.replacement_prescription = prop_value;
                }
                "replacement_verb" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.replacement_verb = prop_value;
                }
                "show_avg" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.show_average = prop_value;
                }
                "Target" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.target = prop_value;
                }
                "Text" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.text = prop_value;
                }
                "units" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.units = prop_value;
                }
                "Zone" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.zone = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'Warmup' tag
pub fn get_properties_cooldown(chunk: &str) -> Cooldown {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Cadence".to_string(),
        "CadenceHigh".to_string(),
        "CadenceLow".to_string(),
        "CadenceResting".to_string(),
        "Duration".to_string(),
        "end_at_road_time".to_string(),
        "pace".to_string(),
        "Pace".to_string(),
        "Power".to_string(),
        "PowerHigh".to_string(),
        "PowerLow".to_string(),
        "replacement_prescription".to_string(),
        "replacement_verb".to_string(),
        "units".to_string(),
        "Zone".to_string(),
    ];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_cooldown_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "Cadence" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence = prop_value;
                }
                "CadenceHigh" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_high = prop_value;
                }
                "CadenceLow" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_low = prop_value;
                }
                "CadenceResting" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_resting = prop_value;
                }
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "end_at_road_time" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.end_at_road_time = prop_value;
                }
                "pace" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.pace = prop_value;
                }
                "Pace" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.pace2 = prop_value;
                }
                "Power" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power = prop_value;
                }
                "PowerHigh" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_high = prop_value;
                }
                "PowerLow" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_low = prop_value;
                }
                "replacement_prescription" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.replacement_prescription = prop_value;
                }
                "replacement_verb" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.replacement_verb = prop_value;
                }
                "units" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.units = prop_value;
                }
                "Zone" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.zone = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'FreeRide' tag
pub fn get_properties_free_ride(chunk: &str) -> FreeRide {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Cadence".to_string(),
        "CadenceHigh".to_string(),
        "CadenceLow".to_string(),
        "Duration".to_string(),
        "FailThresholdDuration".to_string(),
        "FlatRoad".to_string(),
        "ftptest".to_string(),
        "Power".to_string(),
        "ramptest".to_string(),
        "show_avg".to_string(),
    ];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_free_ride_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "Cadence" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence = prop_value;
                }
                "CadenceHigh" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_high = prop_value;
                }
                "CadenceLow" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_low = prop_value;
                }
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "FailThresholdDuration" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.fail_threshold_duration = prop_value;
                }
                "FlatRoad" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.flat_road = prop_value;
                }
                "ftptest" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.ftp_test = prop_value;
                }
                "Power" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power = prop_value;
                }
                "ramptest" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.ramp_test = prop_value;
                }
                "show_avg" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.show_average = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'Freeride' tag
pub fn get_properties_free_ride2(chunk: &str) -> Freeride {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Duration".to_string(),
        "FlatRoad".to_string(),
        "ftptest".to_string(),
    ];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_free_ride2_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "FlatRoad" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.flat_road = prop_value;
                }
                "ftptest" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.ftp_test = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'IntervalsT' tag
pub fn get_properties_intervalst(chunk: &str) -> IntervalsT {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Cadence".to_string(),
        "CadenceHigh".to_string(),
        "CadenceLow".to_string(),
        "CadenceResting".to_string(),
        "FlatRoad".to_string(),
        "OffDuration".to_string(),
        "OffPower".to_string(),
        "OnDuration".to_string(),
        "OnPower".to_string(),
        "OverUnder".to_string(),
        "pace".to_string(),
        "PowerOffHigh".to_string(),
        "PowerOffLow".to_string(),
        "PowerOffZone".to_string(),
        "PowerOnHigh".to_string(),
        "PowerOnLow".to_string(),
        "PowerOnZone".to_string(),
        "Repeat".to_string(),
        "units".to_string(),
    ];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_intervalst_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "Cadence" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence = prop_value;
                }
                "CadenceHigh" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_high = prop_value;
                }
                "CadenceLow" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_low = prop_value;
                }
                "CadenceResting" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_resting = prop_value;
                }
                "FlatRoad" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.flat_road = prop_value;
                }
                "OffDuration" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.off_duration = prop_value;
                }
                "OffPower" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.off_power = prop_value;
                }
                "OnDuration" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.on_duration = prop_value;
                }
                "OnPower" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.on_power = prop_value;
                }
                "OverUnder" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.over_under = prop_value;
                }
                "pace" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.pace = prop_value;
                }
                "PowerOffHigh" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_off_high = prop_value;
                }
                "PowerOffLow" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_off_low = prop_value;
                }
                "PowerOffZone" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.power_off_zone = prop_value;
                }
                "PowerOnHigh" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_on_high = prop_value;
                }
                "PowerOnLow" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_on_low = prop_value;
                }
                "PowerOnZone" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.power_on_zone = prop_value;
                }
                "Repeat" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.repeat = prop_value;
                }
                "units" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.units = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'MaxEffort' tag
pub fn get_properties_max_effort(chunk: &str) -> MaxEffort {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec!["duration".to_string()];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_max_effort_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() - property.len(), chunk.len());
            match property.as_str() {
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'Ramp' tag
pub fn get_properties_ramp(chunk: &str) -> Ramp {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Cadence".to_string(),
        "CadenceResting".to_string(),
        "Duration".to_string(),
        "pace".to_string(),
        "Power".to_string(),
        "PowerHigh".to_string(),
        "PowerLow".to_string(),
        "show_avg".to_string(),
    ];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_ramp_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "Cadence" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence = prop_value;
                }
                "CadenceResting" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.cadence_resting = prop_value;
                }
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "pace" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.pace = prop_value;
                }
                "Power" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power = prop_value;
                }
                "PowerHigh" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_high = prop_value;
                }
                "PowerLow" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power_low = prop_value;
                }
                "show_avg" => {
                    let prop_value = get_property_value_bool(&prop_str);
                    prop_struct.show_average = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of tag properties for 'SolidState' tag
pub fn get_properties_solid_state(chunk: &str) -> SolidState {
    // property strings - .zwo files are really inconsistent, so pay careful attention
    let properties = vec!["duration".to_string(), "power".to_string()];

    // create property struct filled with Nones, replace as properties are found
    let mut prop_struct = new_solid_state_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() - property.len(), chunk.len());
            match property.as_str() {
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "Power" => {
                    let prop_value = get_property_value_float(&prop_str);
                    prop_struct.power = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of properties for 'TextEvent' tag
pub fn get_properties_text_event(chunk: &str) -> TextEvent {
    // property names - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "Duration".to_string(),
        "message".to_string(),
        "TimeOffset".to_string(),
        "timeoffset".to_string(),
    ];

    // create propery struct filled with Nones
    let mut prop_struct = new_text_event_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() - property.len(), chunk.len());
            match property.as_str() {
                "Duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "message" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.message = prop_value;
                }
                "TimeOffset" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.time_offset = prop_value;
                }
                "timeoffset" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.time_offset2 = prop_value;
                }
                _ => {}
            }
        }
    }

    // TODO: if no duration is specified, set it to some sensible default

    return prop_struct;
}

/// gets list of properties for 'TextEvent' tag
pub fn get_properties_text_event2(chunk: &str) -> Textevent {
    // property names - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "distoffset".to_string(),
        "duration".to_string(),
        "message".to_string(),
        "timeoffset".to_string(),
    ];

    // create propery struct filled with Nones
    let mut prop_struct = new_text_event2_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() + property.len(), chunk.len());
            match property.as_str() {
                "distoffset" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.distoffset = prop_value;
                }
                "duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "message" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.message = prop_value;
                }
                "timeoffset" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.time_offset = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// gets list of properties for 'TextEvent' tag
pub fn get_properties_text_notification(chunk: &str) -> TextNotification {
    // property names - .zwo files are really inconsistent, so pay careful attention
    let properties = vec![
        "duration".to_string(),
        "text".to_string(),
        "timeoffset".to_string(),
    ];

    // create propery struct filled with Nones
    let mut prop_struct = new_text_notification_struct();

    // get sub string from property name to end
    for property in properties.iter() {
        let prop_pos = chunk.find(property);
        if prop_pos.is_some() {
            let prop_str = get_substr(&chunk, prop_pos.unwrap() - property.len(), chunk.len());
            match property.as_str() {
                "duration" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.duration = prop_value;
                }
                "text" => {
                    let prop_value = get_property_value_str(&prop_str);
                    prop_struct.text = prop_value;
                }
                "timeoffset" => {
                    let prop_value = get_property_value_integer(&prop_str);
                    prop_struct.time_offset = prop_value;
                }
                _ => {}
            }
        }
    }

    return prop_struct;
}

/// get property value for integer properties
fn get_property_value_integer(chunk: &str) -> Option<i32> {
    let mut value: Option<i32> = None;
    let quote_pos1 = chunk.find("\"");
    if quote_pos1.is_some() {
        let new_sub_str = get_substr(chunk, quote_pos1.unwrap() + 1, chunk.len());
        let quote_pos2 = new_sub_str.find("\"");
        if quote_pos2.is_some() {
            let value_str = get_substr(chunk, quote_pos1.unwrap() + 1, quote_pos2.unwrap() + 2);
            let value_result = value_str.parse::<i32>();
            match value_result {
                Ok(n) => value = Some(n),
                Err(_) => {}
            }
        }
    }
    return value;
}

/// get property value for float properties
fn get_property_value_float(chunk: &str) -> Option<f32> {
    let mut value: Option<f32> = None;
    let quote_pos1 = chunk.find("\"");
    if quote_pos1.is_some() {
        let new_sub_str = get_substr(chunk, quote_pos1.unwrap() + 1, chunk.len());
        let quote_pos2 = new_sub_str.find("\"");
        if quote_pos2.is_some() {
            let value_str = get_substr(chunk, quote_pos1.unwrap() + 1, quote_pos2.unwrap() + 2);
            let value_result = value_str.parse::<f32>();
            match value_result {
                Ok(n) => value = Some(n),
                Err(_) => {}
            }
        }
    }
    return value;
}

/// get property value for string properties
fn get_property_value_str(chunk: &str) -> Option<String> {
    let mut value: Option<String> = None;
    let quote_pos1 = chunk.find("\"");
    if quote_pos1.is_some() {
        let new_sub_str = get_substr(chunk, quote_pos1.unwrap() + 1, chunk.len());
        let quote_pos2 = new_sub_str.find("\"");
        if quote_pos2.is_some() {
            let value_str = get_substr(chunk, quote_pos1.unwrap() + 1, quote_pos2.unwrap() + 2);
            value = Some(value_str);
        }
    }
    return value;
}

/// get property value for boolean properties
fn get_property_value_bool(chunk: &str) -> Option<bool> {
    let mut value: Option<bool> = None;
    let quote_pos1 = chunk.find("\"");
    if quote_pos1.is_some() {
        let new_sub_str = get_substr(chunk, quote_pos1.unwrap() + 1, chunk.len());
        let quote_pos2 = new_sub_str.find("\"");
        if quote_pos2.is_some() {
            let value_str = get_substr(chunk, quote_pos1.unwrap() + 1, quote_pos2.unwrap() + 2);
            match value_str.as_str() {
                "1" => value = Some(true),
                "0" => value = Some(false),
                _ => value = None,
            }
        }
    }
    return value;
}

/// returns a warmup struct with None in all fields, convenience function to initialize a Warmup struct
fn new_warmup_struct() -> Warmup {
    let prop_struct = Warmup {
        cadence: None,
        cadence_high: None,
        cadence_low: None,
        cadence_resting: None,
        duration: None,
        pace: None,
        power: None,
        power_high: None,
        power_low: None,
        quantize: None,
        replacement_prescription: None,
        replacement_verb: None,
        text: None,
        units: None,
        zone: None,
    };
    return prop_struct;
}

/// returns a SteadyState struct with None in all fields, convenience function to initialize a SteadyState struct
fn new_steady_state_struct() -> SteadyState {
    let prop_struct = SteadyState {
        cadence: None,
        cadence_high: None,
        cadence_low: None,
        cadence_resting: None,
        duration: None,
        fail_threshold_duration: None,
        forced_performance_test1: None,
        forced_performance_test2: None,
        never_fails: None,
        off_power: None,
        pace: None,
        power: None,
        power_high: None,
        power_low: None,
        ramp_test: None,
        replacement_prescription: None,
        replacement_verb: None,
        show_average: None,
        target: None,
        text: None,
        units: None,
        zone: None,
    };
    return prop_struct;
}

/// returns a Cooldown struct with None in all fields, convenience function to initialize a Cooldown struct
fn new_cooldown_struct() -> Cooldown {
    let prop_struct = Cooldown {
        cadence: None,
        cadence_high: None,
        cadence_low: None,
        cadence_resting: None,
        duration: None,
        end_at_road_time: None,
        pace: None,
        pace2: None,
        power: None,
        power_high: None,
        power_low: None,
        replacement_prescription: None,
        replacement_verb: None,
        units: None,
        zone: None,
    };
    return prop_struct;
}

/// returns a FreeRide struct with None in all fields, convenience function to initialize a FreeRide struct
fn new_free_ride_struct() -> FreeRide {
    let prop_struct = FreeRide {
        cadence: None,
        cadence_high: None,
        cadence_low: None,
        duration: None,
        fail_threshold_duration: None,
        flat_road: None,
        ftp_test: None,
        power: None,
        ramp_test: None,
        show_average: None,
    };
    return prop_struct;
}

/// returns a Freeride struct with None in all fields, convenience function to initialize a Freeride struct
fn new_free_ride2_struct() -> Freeride {
    let prop_struct = Freeride {
        duration: None,
        flat_road: None,
        ftp_test: None,
    };
    return prop_struct;
}

/// returns a IntervalsT struct with None in all fields, convenience function to initialize a IntervalsT struct
fn new_intervalst_struct() -> IntervalsT {
    let prop_struct = IntervalsT {
        cadence: None,
        cadence_high: None,
        cadence_low: None,
        cadence_resting: None,
        flat_road: None,
        off_duration: None,
        off_power: None,
        on_duration: None,
        on_power: None,
        over_under: None,
        pace: None,
        power_off_high: None,
        power_off_low: None,
        power_off_zone: None,
        power_on_high: None,
        power_on_low: None,
        power_on_zone: None,
        repeat: None,
        units: None,
    };
    return prop_struct;
}

/// returns a MaxEffort struct with None in all fields, convenience function to initialize a MaxEffort struct
fn new_max_effort_struct() -> MaxEffort {
    let prop_struct = MaxEffort { duration: None };
    return prop_struct;
}

/// returns a Ramp struct with None in all fields, convenience function to initialize a Ramp struct
fn new_ramp_struct() -> Ramp {
    let prop_struct = Ramp {
        cadence: None,
        cadence_resting: None,
        duration: None,
        pace: None,
        power: None,
        power_high: None,
        power_low: None,
        show_average: None,
    };
    return prop_struct;
}

/// returns a SolidState struct with None in all fields, convenience function to initialize a SolidState struct
fn new_solid_state_struct() -> SolidState {
    let prop_struct = SolidState {
        duration: None,
        power: None,
    };
    return prop_struct;
}

/// returns a TextEvent struct with None in all fields, convenience function to initialize a TextEvent struct
fn new_text_event_struct() -> TextEvent {
    let prop_struct = TextEvent {
        duration: None,
        message: None,
        time_offset: None,
        time_offset2: None,
        level: None,
        previous_element: None,
    };
    return prop_struct;
}

/// returns a Textevent struct with None in all fields
fn new_text_event2_struct() -> Textevent {
    let prop_struct = Textevent {
        distoffset: None,
        duration: None,
        message: None,
        text_scale: None,
        time_offset: None,
        level: None,
        previous_element: None,
    };
    return prop_struct;
}

/// returns a TextNotification struct with None in all fields
fn new_text_notification_struct() -> TextNotification {
    let prop_struct = TextNotification {
        duration: None,
        text: None,
        time_offset: None,
        level: None,
        previous_element: None,
    };
    return prop_struct;
}

/// gets the indentation level of text events to get the right time offset
pub fn get_tag_level(file_str: &str, start_pos: usize) -> usize {
    let sub_str = get_substr(file_str, 0, start_pos);
    let start_str = "<";
    let end_str = "</";
    let end_str2 = "/>";
    let a = sub_str.matches(start_str).count();
    let b = sub_str.matches(end_str).count();
    let c = sub_str.matches(end_str2).count();
    let difference = a - (2 * b + c); // 2b to remove closing "</" counting as starts
    return difference;
}
