// Math functions for dealing with time series data.

/// generates a vector of desired length stepping up from one value to the next
pub fn int_linspace(start: i32, end: i32, length: usize) -> Vec<i32> {
    let num_steps = length as i32 / (end - start);
    let extra = length as i32 % (end - start);
    let discrete_values: Vec<i32> = (start..end + 1).collect();
    let mut final_vec: Vec<i32> = Vec::new();
    for (i, value) in discrete_values.iter().enumerate() {
        let mut length = num_steps;
        if ((i + 1) as i32) < extra {
            length += 1;
        }
        let mut inter_vec = vec![value.to_owned(); length as usize];
        final_vec.append(&mut inter_vec);
    }

    return final_vec;
}

/// linspace type function using floats
pub fn float_linspace(start: f32, end: f32, length: usize) -> Vec<f32> {
    let step_size = (end - start) / ((length - 1) as f32);
    let mut final_vec = vec![start];
    for i in 1..(length - 1) {
        final_vec.push(((i as f32) * step_size) + start);
    }
    final_vec.push(end);
    return final_vec;
}
