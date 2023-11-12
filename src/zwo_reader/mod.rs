use std::error::Error;
use std::path::PathBuf;

use self::zwo_parse::*;

pub mod zwo_command;
pub mod zwo_parse;

#[derive(Debug, Clone)]
pub struct Workout {
    pub exercise: Vec<ExerciseTag>,
    pub text: Vec<TextTags>,
}

pub fn zwo_read(filepath: PathBuf) -> Result<Workout, Box<dyn Error>> {
    // TODO: could just load from Pathbuf directly??? find out
    let file_contents = file_to_text(filepath.to_string_lossy().into_owned())?;
    let (_, exercise_tags, text_tags) = get_tag_names();
    // find positions of different tags
    let mut chunk_positions = find_top_tags(&file_contents, &exercise_tags);
    let mut text_chunk_positions = find_top_tags(&file_contents, &text_tags);
    chunk_positions.append(&mut text_chunk_positions);
    let mut chunk_sort = chunk_positions.clone();
    chunk_sort.sort(); // sort tags in file order

    let mut workout: Vec<ExerciseTag> = Vec::new();
    let mut workout_text: Vec<TextTags> = Vec::new();

    for i in 0..chunk_sort.len() {
        // process each portion of the file
        let partial_str: String;
        if i == chunk_sort.len() - 1 {
            partial_str = get_substr(&file_contents, chunk_sort[i], file_contents.len());
        } else {
            partial_str = get_substr(&file_contents, chunk_sort[i], chunk_sort[i + 1]);
        }

        let mut tag_type: Option<ExerciseTag> = None;

        // see if sub string is one of the exercise types
        for tag in exercise_tags.iter() {
            let search_str = format!("<{}", tag); // prevents hits on closing tag
            let find_result = partial_str.find(&search_str);
            match find_result {
                Some(_) => {
                    tag_type = Some(exercise_str2enum(tag));
                    break;
                }
                None => tag_type = None,
            }
        }

        // process text based on the exercise type
        if tag_type.is_some() {
            match tag_type.unwrap() {
                ExerciseTag::Warmup(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let warmup_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let warmup_struct = get_properties_warmup(&warmup_str);
                        workout.push(ExerciseTag::Warmup(warmup_struct));
                    }
                }
                ExerciseTag::SteadyState(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let steady_state_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let steady_state_struct = get_properties_steady_state(&steady_state_str);
                        workout.push(ExerciseTag::SteadyState(steady_state_struct));
                    }
                }
                ExerciseTag::Cooldown(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let cooldown_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let cooldown_struct = get_properties_cooldown(&cooldown_str);
                        workout.push(ExerciseTag::Cooldown(cooldown_struct));
                    }
                }
                ExerciseTag::FreeRide(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let free_ride_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let free_ride_struct = get_properties_free_ride(&free_ride_str);
                        workout.push(ExerciseTag::FreeRide(free_ride_struct));
                    }
                }
                ExerciseTag::Freeride(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let free_ride2_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let free_ride2_struct = get_properties_free_ride2(&free_ride2_str);
                        workout.push(ExerciseTag::Freeride(free_ride2_struct));
                    }
                }
                ExerciseTag::IntervalsT(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let intervalst_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let intervalst_struct = get_properties_intervalst(&intervalst_str);
                        workout.push(ExerciseTag::IntervalsT(intervalst_struct));
                    }
                }
                ExerciseTag::MaxEffort(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let max_effort_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let max_effort_struct = get_properties_max_effort(&max_effort_str);
                        workout.push(ExerciseTag::MaxEffort(max_effort_struct));
                    }
                }
                ExerciseTag::Ramp(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let ramp_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let ramp_struct = get_properties_ramp(&ramp_str);
                        workout.push(ExerciseTag::Ramp(ramp_struct));
                    }
                }
                ExerciseTag::SolidState(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let solid_state_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let solid_state_struct = get_properties_solid_state(&solid_state_str);
                        workout.push(ExerciseTag::SolidState(solid_state_struct));
                    }
                }
                _ => {}
            }
        }

        // check if sub string is one of the text events
        let mut text_tag_type: Option<TextTags> = None;
        for tag in text_tags.iter() {
            let find_result = partial_str.find(tag);
            match find_result {
                Some(_) => {
                    text_tag_type = Some(text_str2enum(tag));
                    break;
                }
                None => text_tag_type = None,
            }
        }

        // process text based on the text event type
        if text_tag_type.is_some() {
            match text_tag_type.unwrap() {
                TextTags::TextEvent(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let text_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let mut text_struct = get_properties_text_event(&text_str);

                        // get text level - need to reference file string up to this point
                        text_struct.level = Some(get_tag_level(&file_contents, chunk_sort[i]));
                        // I don't expect this one to fail, but we will see.

                        if text_struct.level.is_some() {
                            if text_struct.level.unwrap() > 2 {
                                // TODO: double check that sequential texts don't stack off each other
                                text_struct.previous_element = Some(workout.len());
                            }
                        }
                        workout_text.push(TextTags::TextEvent(text_struct));
                    }
                }
                TextTags::TextEvent2(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let text_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let mut text_struct = get_properties_text_event2(&text_str);

                        // get text level - need to reference file string up to this point
                        text_struct.level = Some(get_tag_level(&file_contents, chunk_sort[i]));
                        // I don't expect this one to fail, but we will see.

                        if text_struct.level.is_some() {
                            if text_struct.level.unwrap() > 2 {
                                // TODO: double check that sequential texts don't stack off each other
                                text_struct.previous_element = Some(workout.len());
                            }
                        }
                        workout_text.push(TextTags::TextEvent2(text_struct));
                    }
                }
                TextTags::TextNotification(_) => {
                    let end_pos = partial_str.find(">");
                    if end_pos.is_some() {
                        let text_str = get_substr(&partial_str, 0, end_pos.unwrap());
                        let mut text_struct = get_properties_text_notification(&text_str);

                        // get text level - need to reference file string up to this point
                        text_struct.level = Some(get_tag_level(&file_contents, chunk_sort[i]));
                        // I don't expect this one to fail, but we will see.

                        if text_struct.level.is_some() {
                            if text_struct.level.unwrap() > 2 {
                                // TODO: double check that sequential texts don't stack off each other
                                text_struct.previous_element = Some(workout.len());
                            }
                        }
                        workout_text.push(TextTags::TextNotification(text_struct));
                    }
                }
                _ => {}
            }
        }
    }

    let workout_struct = Workout {
        exercise: workout,
        text: workout_text,
    };

    return Ok(workout_struct);
}
