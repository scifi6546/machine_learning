use std::f64::consts::{self, PI};
pub fn hann(data: &[f64]) -> Vec<f64> {
    let m = data.len() as f64;
    data.iter()
        .enumerate()
        .map(|(n, value)| (n as f64, value))
        .map(|(n, value)| ((PI * n / (m - 1.)).sin()) * value)
        .collect()
}
