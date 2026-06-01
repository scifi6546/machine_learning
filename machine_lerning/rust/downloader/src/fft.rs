use fftw::{
    array::AlignedVec,
    plan::{R2CPlan, R2CPlan64, R2RPlan, R2RPlan64},
    types::{Flag, R2RKind},
};
pub mod window;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imaginary: f64,
}
impl Complex {
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imaginary * self.imaginary).sqrt()
    }
}
pub fn make_fft(data: &[f64]) -> Vec<Complex> {
    let n = data.len();
    let mut aligned_input = AlignedVec::new(n);

    for (index, value) in data.iter().enumerate() {
        aligned_input[index] = *value;
    }

    let mut plan: R2CPlan64 = R2CPlan::aligned(&[n], Flag::MEASURE).expect("failed to make plan");

    let mut aligned_output = AlignedVec::new(n / 2 + 1);

    plan.r2c(&mut aligned_input, &mut aligned_output).unwrap();
    aligned_output
        .iter()
        .map(|complex| Complex {
            real: complex.re,
            imaginary: complex.im,
        })
        .collect()
}
pub fn get_frequencies(frequency_data: &[Complex], sample_rate: f64) -> Vec<f64> {
    (0..frequency_data.len())
        .map(|index| 2. * sample_rate * frequency_data.len() as f64 / index as f64)
        .collect::<Vec<_>>()
}
pub struct Data {
    data: Vec<f64>,
    sample_rate: f64,
}
impl Data {
    pub fn new(data: Vec<f64>, sample_rate: f64) -> Self {
        Self { data, sample_rate }
    }
    /// computes an fft with the given window length
    pub fn compute_short_term_fft(&self, window_length: usize) -> ShortTermFFT {
        let number_windows = self.data.len() / window_length;
        let mut fft_window_data = Vec::new();
        let mut frequency_map = None;
        for i in 0..number_windows {
            let window_data = &self.data[(i * window_length)..((i + 1) * window_length)];
            let window_data = window::hann(window_data);
            let mut sample_data = make_fft(&window_data);
            if frequency_map.is_none() {
                frequency_map = Some(get_frequencies(&sample_data, self.sample_rate))
            }

            fft_window_data.append(&mut sample_data);
        }

        ShortTermFFT {
            data: fft_window_data,
            frequency_map: frequency_map.expect("should be populated if self.data.len()>0"),
            sample_rate: self.sample_rate,
            number_windows,
        }
    }
}
pub enum ColorTransformFunction {
    Constant,
    Log10,
}
impl ColorTransformFunction {
    fn transform(&self, data: f64) -> f64 {
        match self {
            Self::Constant => data,
            Self::Log10 => data.log10(),
        }
    }
}
pub struct ShortTermFFTPlotOptions<'a> {
    pub title: &'a str,
    pub color_transform_fn: ColorTransformFunction,
}
impl<'a> ShortTermFFTPlotOptions<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            color_transform_fn: ColorTransformFunction::Constant,
        }
    }
    pub fn color_transform(mut self, color_transform_function: ColorTransformFunction) -> Self {
        self.color_transform_fn = color_transform_function;
        self
    }
}
pub struct ShortTermFFT {
    data: Vec<Complex>,
    frequency_map: Vec<f64>,
    sample_rate: f64,
    number_windows: usize,
}
impl ShortTermFFT {
    pub fn plot(&self, options: ShortTermFFTPlotOptions<'_>) {
        use plotters::prelude::*;
        let save_path = format!("./{}_spectrogram.png", options.title);
        let root = BitMapBackend::new(&save_path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let max_magnitude = self.data.iter().fold(f64::MIN, |acc, x| {
            let magnitude = options.color_transform_fn.transform(x.magnitude());
            if magnitude > acc { magnitude } else { acc }
        });

        let frequency_min = self
            .frequency_map
            .iter()
            .copied()
            .fold(f64::MAX, |acc, x| if x < acc { x } else { acc });
        let frequency_max = self
            .frequency_map
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .fold(f64::MIN, |acc, x| if x > acc { x } else { acc });

        let mut chart = ChartBuilder::on(&root)
            .caption(options.title, 80)
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0.0..(self.number_windows as f64), 0.0..frequency_max)
            .unwrap();
        println!("created chart");

        chart.configure_mesh().draw().unwrap();

        println!("frequency_map len: {}", self.frequency_map.len());
        chart
            .draw_series(
                (0..(self.frequency_map.len()))
                    .flat_map(|y| (0..(self.number_windows)).map(move |x| (x, y)))
                    .map(|(x, y)| {
                        let v = options
                            .color_transform_fn
                            .transform(self.data[y + x * self.frequency_map.len()].magnitude())
                            / max_magnitude;
                        let v = if v.is_nan() || v.is_infinite() || v.is_sign_negative() {
                            0.
                        } else {
                            v
                        };
                        let y_bottom = if y > 0 { self.frequency_map[y - 1] } else { 0. };
                        let y_top = self.frequency_map[y];
                        Rectangle::new(
                            [(x as f64, y_bottom), (x as f64 + 1., y_top)],
                            HSLColor(0.5, 0., v).filled(),
                        )
                    }),
            )
            .expect("failed to draw spectrogram");
        root.present().expect("failed to draw spectrogram");
        {
            let root = BitMapBackend::new("frequency_bins.png", (1024, 768)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            let mut chart = ChartBuilder::on(&root)
                .caption("Frequency bins for spectrogram", 80)
                .margin(5)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .build_cartesian_2d(0.0..frequency_max, 0.0..(self.frequency_map.len() as f64))
                .unwrap();
            chart
                .configure_mesh()
                .x_desc("Frequency (hz)")
                .y_desc("x index")
                .draw()
                .unwrap();
            chart
                .draw_series(LineSeries::new(
                    self.frequency_map
                        .iter()
                        .enumerate()
                        .map(|(i, v)| (i as f64, *v)),
                    &RED,
                ))
                .unwrap();
            chart
                .draw_series(
                    self.frequency_map
                        .iter()
                        .enumerate()
                        .map(|(i, v)| Circle::new((*v, i as f64), 5., RED.filled())),
                )
                .unwrap();
        }
    }
    pub fn save_c_format<P: AsRef<std::path::Path>>(&self, metadata_path: P, spectrogram_path: P) {
        use std::fs::File;
        use std::io::prelude::*;
        println!(
            "saving metadata to {}",
            metadata_path.as_ref().as_os_str().to_str().unwrap()
        );
        let width = self.number_windows;
        let height = self.frequency_map.len();
        let metadata_contents = format!("{} {}", width, height);
        let mut file = File::create(metadata_path).unwrap();
        let number_bytes_written = file.write(metadata_contents.as_bytes()).unwrap();
        assert_eq!(metadata_contents.len(), number_bytes_written);
        let mut binary_file = File::create(spectrogram_path).unwrap();
        let saved_file = self
            .data
            .iter()
            .flat_map(|v| v.magnitude().to_le_bytes())
            .collect::<Vec<_>>();
        binary_file
            .write_all(&saved_file)
            .expect("failed to save file");
    }
}
pub fn plot_window() {
    use plotters::prelude::*;
    let root = BitMapBackend::new("./hann.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let n = 100usize;
    let caption = format!("{} points hann filter", n);
    let mut chart = ChartBuilder::on(&root)
        .caption("hann window", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0f64..(n as f64 - 1.), 0.0..1.0)
        .unwrap();
    chart.configure_mesh().draw().unwrap();
    let data = vec![1.0f64; n];
    let hann_data = window::hann(&data);
    chart
        .draw_series(LineSeries::new(
            hann_data
                .iter()
                .enumerate()
                .map(|(index, v)| (index as f64, *v as f64)),
            &RED,
        ))
        .unwrap()
        .label("trace");
    root.present().expect("failed to draw");
}
