pub fn analyzer(
    wav_file: &mut audio_analyzer_core::prelude::TestData,
    sample_rate: u32,
) -> Vec<Vec<Option<f64>>> {
    use crate::presets::*;
    use audio_analyzer_core::data::RawDataStreamLayer as _;
    let sample_rate = sample_rate as f64;
    let hop_size = {
        let abstract_input_node_out_37_1 = sample_rate;
        let tmp_expr_nodes_18 = abstract_input_node_out_37_1.clone();
        let expr_nodes_out_18_0 = {
            let x = tmp_expr_nodes_18;
            x * 50f64 / 1000f64
        };
        let tmp_expr_nodes_3 = expr_nodes_out_18_0.clone();
        let expr_nodes_out_3_0 = {
            let x = tmp_expr_nodes_3;
            round(x / 16f64) * 16f64
        };
        let tmp_expr_nodes_2 = expr_nodes_out_3_0.clone();
        let expr_nodes_out_2_0 = {
            let x = tmp_expr_nodes_2;
            round(x / 10f64)
        };
        expr_nodes_out_2_0
    };
    let hop_size = hop_size as usize;
    let mut cycle_buffer_node_4 = ndarray::Array1::<f64>::zeros(0);
    let mut stft_layer_5 = audio_analyzer_core::prelude::ToSpectrogramLayer::new(
        audio_analyzer_core::prelude::FftConfig::default(),
    );
    let mut stft_layer_5_fft_size = 400usize;
    let mut stft_layer_5_hop_size = 160usize;
    let mut ifft_1_fft_size = 400;
    let mut ifft_1 = {
        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_inverse(ifft_1_fft_size);
        fft
    };
    let mut ifft_1_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); ifft_1_fft_size];
    let mut return_data = Vec::new();
    let mut buffer = Vec::new();
    while let Some(frame) = wav_file.try_recv() {
        buffer.extend(frame);
        while buffer.len() >= hop_size {
            let abstract_input_node_out_37_0: ndarray::Array1<f64> =
                buffer.drain(..hop_size).collect::<ndarray::Array1<_>>();
            if abstract_input_node_out_37_0.len() == 0 {
                break;
            }
            let abstract_input_node_out_37_1 = sample_rate;
            let tmp_expr_nodes_18 = abstract_input_node_out_37_1.clone();
            let expr_nodes_out_18_0 = {
                let x = tmp_expr_nodes_18;
                x * 50f64 / 1000f64
            };
            let tmp_expr_nodes_3 = expr_nodes_out_18_0.clone();
            let expr_nodes_out_3_0 = {
                let x = tmp_expr_nodes_3;
                round(x / 16f64) * 16f64
            };
            let cycle_buffer_node_out_4_0 = {
                let extended = ndarray::concatenate(
                    ndarray::Axis(0),
                    &[
                        cycle_buffer_node_4.view(),
                        abstract_input_node_out_37_0.view(),
                    ],
                )
                .unwrap();
                let new_len = extended.len() as f64;
                if new_len > expr_nodes_out_3_0 {
                    let diff = (new_len - expr_nodes_out_3_0) as usize;
                    let new_buffer = extended.slice(ndarray::s![diff..]);
                    cycle_buffer_node_4 = new_buffer.to_owned();
                    assert!(new_buffer.len() as f64 == expr_nodes_out_3_0);
                } else {
                    cycle_buffer_node_4 = extended.to_owned();
                }
                cycle_buffer_node_4.clone()
            };
            let tmp_expr_nodes_2 = expr_nodes_out_3_0.clone();
            let expr_nodes_out_2_0 = {
                let x = tmp_expr_nodes_2;
                round(x / 10f64)
            };
            {
                let expr_nodes_out_3_0 = expr_nodes_out_3_0 as usize;
                let expr_nodes_out_2_0 = expr_nodes_out_2_0 as usize;
                if stft_layer_5_fft_size != expr_nodes_out_3_0
                    || stft_layer_5_hop_size != expr_nodes_out_2_0
                {
                    stft_layer_5_fft_size = expr_nodes_out_3_0;
                    stft_layer_5_hop_size = expr_nodes_out_2_0;
                    stft_layer_5 = audio_analyzer_core::prelude::ToSpectrogramLayer::new(
                        audio_analyzer_core::prelude::FftConfig {
                            fft_size: expr_nodes_out_3_0,
                            hop_size: expr_nodes_out_2_0,
                        },
                    );
                }
            }
            let stft_layer_out_5_0 = {
                let stft_layer_out_5_0 = match stft_layer_5
                    .through_inner(&cycle_buffer_node_out_4_0.clone().to_vec())
                    .unwrap()
                    .first()
                {
                    Some(v) => v.to_owned(),
                    None => continue,
                };
                assert!(!stft_layer_out_5_0.is_empty());
                stft_layer_out_5_0
            };
            let tmp_expr_nodes_7 = stft_layer_out_5_0.clone();
            let expr_nodes_out_7_0 = {
                tmp_expr_nodes_7
                    .into_iter()
                    .map(|num_complex::Complex { re: x, im: y }| 20.0 * log_10f64(abs(x)) as f64)
                    .collect::<ndarray::Array1<f64>>()
            };
            let tmp_expr_nodes_0 = expr_nodes_out_7_0.clone();
            let expr_nodes_out_0_0 = {
                tmp_expr_nodes_0
                    .into_iter()
                    .map(|x| {
                        let (a, b) = (x, 0f64);
                        num_complex::Complex::new(a as f64, b as f64)
                    })
                    .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
            };
            if ifft_1_fft_size != expr_nodes_out_0_0.len() {
                ifft_1_fft_size = expr_nodes_out_0_0.len();
                ifft_1 = {
                    let mut planner = rustfft::FftPlanner::new();
                    let fft = planner.plan_fft_inverse(ifft_1_fft_size);
                    fft
                };
                ifft_1_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); ifft_1_fft_size];
            }
            let ifft_out_1_0 = {
                let mut ifft_out_1_0 = expr_nodes_out_0_0.clone().to_vec();
                ifft_1.process_with_scratch(
                    ifft_out_1_0.as_mut_slice(),
                    ifft_1_scratch_buf.as_mut_slice(),
                );
                ifft_out_1_0
                    .into_iter()
                    .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
            };
            let tmp_expr_nodes_10 = ifft_out_1_0.clone();
            let expr_nodes_out_10_0 = {
                tmp_expr_nodes_10
                    .into_iter()
                    .map(|num_complex::Complex { re: x, im: y }| x as f64)
                    .collect::<ndarray::Array1<f64>>()
            };
            let lifter_out_12_0 = {
                let mut quefrency = expr_nodes_out_10_0.clone();
                let index = 15usize;
                for i in 0..quefrency.len() {
                    if i < index || i >= quefrency.len() - index {
                        quefrency[i] = 0.0;
                    }
                }
                quefrency
            };
            return_data.push(
                lifter_out_12_0
                    .into_iter()
                    .map(|x| {
                        if x.is_nan() {
                            None
                        } else if x.is_infinite() {
                            None
                        } else {
                            Some(x)
                        }
                    })
                    .collect::<Vec<Option<f64>>>(),
            );
        }
    }
    return return_data;
}
