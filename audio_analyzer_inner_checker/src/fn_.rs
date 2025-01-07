pub fn analyzer(wav_file: &mut audio_analyzer_core::prelude::TestData, sample_rate: u32) {
    use crate::presets::*;
    use audio_analyzer_core::data::RawDataStreamLayer as _;
    let sample_rate = sample_rate as f64;
    let mut enumerate_iter_node_19: ndarray::Array1<f64> =
        (0..10).step_by(1).map(|x| x as f64).collect();
    let mut enumerate_iter_node_19_state = (0, 1, 10);
    let mut cycle_buffer_node_4 = ndarray::Array1::<f64>::zeros(0);
    let mut lpc_node_30_lpc_order = 10;
    let mut enumerate_iter_node_22: ndarray::Array1<f64> =
        (0..10).step_by(1).map(|x| x as f64).collect();
    let mut enumerate_iter_node_22_state = (0, 1, 10);
    let mut fft_17_fft_size = 400;
    let mut fft_17 = {
        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_17_fft_size);
        fft
    };
    let mut fft_17_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); fft_17_fft_size];
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
    loop {
        let out_37_0: ndarray::Array1<f64> = wav_file
            .try_recv()
            .unwrap()
            .into_iter()
            .map(|v| v as f64)
            .collect();
        if out_37_0.len() == 0 {
            break;
        }
        println!("{:?}", out_37_0);
        let out_37_1 = sample_rate;
        let out_19_0 = {
            let (
                enumerate_iter_node_19_start,
                enumerate_iter_node_19_step,
                enumerate_iter_node_19_end,
            ) = (0usize as usize, 1usize as usize, 10usize as usize);
            if enumerate_iter_node_19_state
                != (
                    enumerate_iter_node_19_start,
                    enumerate_iter_node_19_step,
                    enumerate_iter_node_19_end,
                )
            {
                enumerate_iter_node_19_state = (
                    enumerate_iter_node_19_start,
                    enumerate_iter_node_19_step,
                    enumerate_iter_node_19_end,
                );
                enumerate_iter_node_19 = (enumerate_iter_node_19_start..enumerate_iter_node_19_end)
                    .step_by(enumerate_iter_node_19_step)
                    .map(|x| x as f64)
                    .collect();
            }
            enumerate_iter_node_19.clone()
        };
        let tmp_expr_nodes_18 = out_37_1.clone();
        let out_18_0 = {
            let x = tmp_expr_nodes_18;
            x * 50f64 / 1000f64
        };
        let tmp_expr_nodes_3 = out_18_0.clone();
        let out_3_0 = {
            let x = tmp_expr_nodes_3;
            round(x / 16f64) * 16f64
        };
        let tmp_expr_nodes_2 = out_3_0.clone();
        let out_2_0 = {
            let x = tmp_expr_nodes_2;
            round(x / 10f64)
        };
        let out_4_0 = {
            let extended = ndarray::concatenate(
                ndarray::Axis(0),
                &[cycle_buffer_node_4.view(), out_37_0.view()],
            )
            .unwrap();
            let new_len = extended.len() as f64;
            if new_len > out_3_0 {
                let diff = (new_len - out_3_0) as usize;
                let new_buffer = extended.slice(ndarray::s![diff..]);
                cycle_buffer_node_4 = new_buffer.to_owned();
                assert!(new_buffer.len() as f64 == out_3_0);
            } else {
                cycle_buffer_node_4 = extended.to_owned();
            }
            cycle_buffer_node_4.clone()
        };
        let tmp_expr_nodes_36 = out_4_0.clone();
        let out_36_0 = {
            tmp_expr_nodes_36
                .iter()
                .map(|&x| (float(x > 0.0001) * x) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        if lpc_node_30_lpc_order != 100usize {
            lpc_node_30_lpc_order = 100usize;
        }
        let out_30_0 =
            linear_predictive_coding::calc_lpc_by_levinson_durbin(out_36_0.view(), 100usize);
        let tmp_expr_nodes_34 = out_30_0.clone();
        let out_34_0 = {
            tmp_expr_nodes_34
                .iter()
                .map(|&x| (x * 2f64) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let out_22_0 = {
            let (
                enumerate_iter_node_22_start,
                enumerate_iter_node_22_step,
                enumerate_iter_node_22_end,
            ) = (0usize as usize, 1usize as usize, out_3_0 as usize);
            if enumerate_iter_node_22_state
                != (
                    enumerate_iter_node_22_start,
                    enumerate_iter_node_22_step,
                    enumerate_iter_node_22_end,
                )
            {
                enumerate_iter_node_22_state = (
                    enumerate_iter_node_22_start,
                    enumerate_iter_node_22_step,
                    enumerate_iter_node_22_end,
                );
                enumerate_iter_node_22 = (enumerate_iter_node_22_start..enumerate_iter_node_22_end)
                    .step_by(enumerate_iter_node_22_step)
                    .map(|x| x as f64)
                    .collect();
            }
            enumerate_iter_node_22.clone()
        };
        let tmp_expr_nodes_23 = {
            out_22_0
                .into_iter()
                .map(|v| (out_3_0.clone(), v))
                .collect::<ndarray::Array1<(f64, f64)>>()
        }
        .clone();
        let out_23_0 = {
            tmp_expr_nodes_23
                .into_iter()
                .map(|(x, y)| 0.5 * (1.0 - cos(2.0 * pi() * x / y)) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let tmp_expr_nodes_25 = {
            let is_first_longer = out_4_0.len() as f64 > out_23_0.len() as f64;
            if is_first_longer {
                out_4_0
                    .clone()
                    .slice_move(ndarray::s![..out_23_0.len()])
                    .into_iter()
                    .zip(out_23_0.clone().into_iter())
                    .collect::<ndarray::Array1<(f64, f64)>>()
            } else {
                out_4_0
                    .clone()
                    .slice_move(ndarray::s![..out_4_0.len()])
                    .into_iter()
                    .zip(out_4_0.clone().into_iter())
                    .collect::<ndarray::Array1<(f64, f64)>>()
            }
        }
        .clone();
        let out_25_0 = {
            tmp_expr_nodes_25
                .into_iter()
                .map(|(x, y)| {
                    let (a, b) = (x * y, 0f64);
                    num_complex::Complex::new(a as f64, b as f64)
                })
                .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
        };
        if fft_17_fft_size != out_25_0.len() {
            fft_17_fft_size = out_25_0.len();
            fft_17 = {
                let mut planner = rustfft::FftPlanner::new();
                let fft = planner.plan_fft_forward(fft_17_fft_size);
                fft
            };
            fft_17_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); fft_17_fft_size];
        }
        let out_17_0 = {
            let mut out_17_0 = out_25_0.clone().to_vec();
            fft_17.process_with_scratch(out_17_0.as_mut_slice(), fft_17_scratch_buf.as_mut_slice());
            out_17_0
                .into_iter()
                .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
        };
        let tmp_expr_nodes_27 = out_17_0.clone();
        let out_27_0 = {
            tmp_expr_nodes_27
                .into_iter()
                .map(|num_complex::Complex { re: x, im: y }| {
                    let (a, b) = (x, y);
                    (a as f64, b as f64)
                })
                .collect::<ndarray::Array1<(f64, f64)>>()
        };
        let out_5_0 = stft_layer_5
            .through_inner(&out_4_0.clone().to_vec())
            .unwrap()
            .first()
            .unwrap()
            .to_owned();
        {
            let out_3_0 = out_3_0 as usize;
            let out_2_0 = out_2_0 as usize;
            if stft_layer_5_fft_size != out_3_0 || stft_layer_5_hop_size != out_2_0 {
                stft_layer_5_fft_size = out_3_0;
                stft_layer_5_hop_size = out_2_0;
                stft_layer_5 = audio_analyzer_core::prelude::ToSpectrogramLayer::new(
                    audio_analyzer_core::prelude::FftConfig {
                        fft_size: out_3_0,
                        hop_size: out_2_0,
                    },
                );
            }
        }
        let tmp_expr_nodes_26 = out_5_0.clone();
        let out_26_0 = {
            tmp_expr_nodes_26
                .into_iter()
                .map(|num_complex::Complex { re: x, im: y }| {
                    let (a, b) = (x, y);
                    (a as f64, b as f64)
                })
                .collect::<ndarray::Array1<(f64, f64)>>()
        };
        let tmp_expr_nodes_21 = out_19_0.clone();
        let out_21_0 = {
            tmp_expr_nodes_21
                .iter()
                .map(|&x| (x / 10f64) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let tmp_expr_nodes_7 = out_5_0.clone();
        let out_7_0 = {
            tmp_expr_nodes_7
                .into_iter()
                .map(|num_complex::Complex { re: x, im: y }| 20.0 * log_10f64(abs(x)) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let tmp_expr_nodes_0 = out_7_0.clone();
        let out_0_0 = {
            tmp_expr_nodes_0
                .into_iter()
                .map(|x| {
                    let (a, b) = (x, 0f64);
                    num_complex::Complex::new(a as f64, b as f64)
                })
                .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
        };
        if ifft_1_fft_size != out_0_0.len() {
            ifft_1_fft_size = out_0_0.len();
            ifft_1 = {
                let mut planner = rustfft::FftPlanner::new();
                let fft = planner.plan_fft_inverse(ifft_1_fft_size);
                fft
            };
            ifft_1_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); ifft_1_fft_size];
        }
        let out_1_0 = {
            let mut out_1_0 = out_0_0.clone().to_vec();
            ifft_1.process_with_scratch(out_1_0.as_mut_slice(), ifft_1_scratch_buf.as_mut_slice());
            out_1_0
                .into_iter()
                .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
        };
        let tmp_expr_nodes_10 = out_1_0.clone();
        let out_10_0 = {
            tmp_expr_nodes_10
                .into_iter()
                .map(|num_complex::Complex { re: x, im: y }| x as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let out_12_0 = {
            let mut quefrency = out_10_0.clone();
            let index = 15usize;
            for i in 0..quefrency.len() {
                if i < index || i >= quefrency.len() - index {
                    quefrency[i] = 0.0;
                }
            }
            quefrency
        };
        let tmp_expr_nodes_15 = out_12_0.clone();
        let out_15_0 = {
            tmp_expr_nodes_15
                .iter()
                .map(|&x| (x / 10000f64) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
    }
}
