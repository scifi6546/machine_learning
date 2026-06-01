use core::f64;

use iris_client::{WaveformClient, WaveformFetchInfo};
use std::f64::consts::PI;

use prelude::chrono::{TimeDelta, prelude::*};
mod fft;
use earthquakes::{connection::ConnectionBuilder, query::EventQuery};
use fft::Complex;
fn plot_fft_data(input: &[f64], frequency_data: &[Complex], title: &str, sample_rate: f32) {
    use plotters::prelude::*;
    let raw_title = format!("{}_data.png", title);
    let root = BitMapBackend::new(&raw_title, (640, 480)).into_drawing_area();
    let input_min = input
        .iter()
        .fold(f64::MAX, |acc, x| if *x < acc { *x } else { acc }) as f32;
    let input_max = input
        .iter()
        .fold(f64::MIN, |acc, x| if *x > acc { *x } else { acc }) as f32;
    let mut chart = ChartBuilder::on(&root)
        .caption("data", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0f32..(input.len() as f32), input_min..input_max)
        .unwrap();
    root.fill(&WHITE).unwrap();
    chart
        .configure_mesh()
        .x_desc("Sample Number")
        .y_desc("Sensor Unit")
        .draw()
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            input
                .iter()
                .enumerate()
                .map(|(index, v)| (index as f32, *v as f32)),
            &RED,
        ))
        .unwrap()
        .label("trace")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
    root.present().expect("failed to draw");
    let x_data = fft::get_frequencies(frequency_data, sample_rate as f64)
        .iter()
        .copied()
        .map(|f| f as f32)
        .collect::<Vec<_>>();
    let frequency_max = frequency_data
        .iter()
        .fold(f64::MIN, |acc, x| if x.real > acc { x.real } else { acc })
        as f32;
    let frequency_min = frequency_data
        .iter()
        .fold(f64::MAX, |acc, x| if x.real < acc { x.real } else { acc })
        as f32;

    let imaginary_max =
        frequency_data.iter().fold(
            f64::MIN,
            |acc, x| if x.imaginary > acc { x.imaginary } else { acc },
        ) as f32;
    let imaginary_min =
        frequency_data.iter().fold(
            f64::MAX,
            |acc, x| if x.imaginary < acc { x.imaginary } else { acc },
        ) as f32;

    let magnitudes = frequency_data
        .iter()
        .map(|v| v.magnitude())
        .collect::<Vec<_>>();
    let magnitudes_max = magnitudes
        .iter()
        .fold(f64::MIN, |acc, x| if *x > acc { *x } else { acc }) as f32;
    let magnitudes_min = magnitudes
        .iter()
        .fold(f64::MAX, |acc, x| if *x < acc { *x } else { acc }) as f32;
    let total_max = [frequency_max, imaginary_max, magnitudes_max]
        .iter()
        .fold(f32::MIN, |acc, x| if *x > acc { *x } else { acc });
    let total_min = [frequency_min, imaginary_min, magnitudes_min]
        .iter()
        .fold(f32::MAX, |acc, x| if *x < acc { *x } else { acc });

    let frequency_title = format!("{}_frequency.png", title);
    let root = BitMapBackend::new(&frequency_title, (1040, 880)).into_drawing_area();
    let mut chart = ChartBuilder::on(&root)
        .caption("data", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0f32..(3000.), total_min..total_max)
        .unwrap();
    root.fill(&WHITE).unwrap();
    chart
        .configure_mesh()
        .x_desc("Frequency (hz)")
        .y_desc("Power")
        .draw()
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            frequency_data
                .iter()
                .zip(x_data.iter().cloned())
                .map(|(v, x)| (x, v.real as f32)),
            &RED,
        ))
        .unwrap()
        .label("real");

    chart
        .draw_series(LineSeries::new(
            frequency_data
                .iter()
                .zip(x_data.iter().cloned())
                .map(|(v, x)| (x, v.imaginary as f32)),
            &BLUE,
        ))
        .unwrap()
        .label("imaginary");

    chart
        .draw_series(LineSeries::new(
            magnitudes
                .iter()
                .cloned()
                .zip(x_data.iter().cloned())
                .map(|(v, x)| (x, v as f32)),
            &GREEN,
        ))
        .unwrap()
        .label("magnitude");

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
    root.present().expect("failed to generate image");
}
fn main() {
    let mut client = WaveformClient::new().unwrap();
    let connection = ConnectionBuilder::default().connect().unwrap();
    let query = EventQuery::default().with_event_name("018fcnsk91".to_string());
    let events = connection.query(query).unwrap();
    println!("{:#?}", events);
    let stations = [("AK", "RC01"), ("AK", "FIRE"), ("AK", "GHO"), ("AK", "SWD")];
    for event in events.iter() {
        let start_time = event.time - TimeDelta::hours(24);
        let end_time = event.time + TimeDelta::hours(1);
        for (network, station) in stations {
            let fetch_info = WaveformFetchInfo {
                network: network.to_string(),
                station: station.to_string(),
                channel_select: "BH*".to_string(),
                start_time,
                end_time,
            };
            let waveform = client.fetch(&fetch_info).unwrap();
            for trace in waveform.traces() {
                println!("{}", trace.channel());
                let data_f64 = trace.data().iter().map(|v| *v as f64).collect::<Vec<_>>();
                let spectrogram =
                    fft::Data::new(data_f64, trace.sampling_rate()).compute_short_term_fft(100);
                let title = format!("ak{}_{}_{}", event.event_name, station, trace.channel());
                println!("{}", title);
                spectrogram.plot(
                    fft::ShortTermFFTPlotOptions::new(&title)
                        .color_transform(fft::ColorTransformFunction::Log10),
                );
                let metadata_file = format!("./output_data/{}_metadata.txt", title);
                let spectrogram_file = format!("./output_data/{}_spectrogram.bin", title);
                spectrogram.save_c_format(&metadata_file, &spectrogram_file);
            }
            println!("{}", waveform.traces().len())
        }
    }
}
