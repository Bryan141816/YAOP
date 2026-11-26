pub fn str_arr_to_f32(str_arr: &[&str]) -> Vec<f32> {
    str_arr
        .iter()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect()
}