pub fn str_arr_to_f32(str_arr: &[&str]) -> Vec<f32> {
    let mut arr = Vec::new();
    for str in str_arr {
        let num = str.parse::<f32>().expect("Failed to parse string as f32");
        arr.push(num);
    }
    arr
} 