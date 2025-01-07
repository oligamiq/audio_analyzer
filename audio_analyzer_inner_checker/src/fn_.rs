pub fn analyzer(wav_file: &mut audio_analyzer_core::prelude::TestData, sample_rate: u32) {
    use audio_analyzer_core::data::RawDataStreamLayer as _;
    let sample_rate = sample_rate;
    let mut FFT_17_fft_size = 400;
    let mut FFT_17 = {
        let planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_17_fft_size);
        fft
    };
    let mut FFT_17_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); FFT_17_fft_size];
    let mut STFTLayer_5 = audio_analyzer_core::prelude::ToSpectrogramLayer::new(
        audio_analyzer_core::prelude::FftConfig::default(),
    );
    let mut STFTLayer_5_fft_size = 400;
    let mut STFTLayer_5_hop_size = 160;
    let mut IFFT_1_fft_size = 400;
    let mut IFFT_1 = {
        let planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_inverse(IFFT_1_fft_size);
        fft
    };
    let mut IFFT_1_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); IFFT_1_fft_size];
    let mut CycleBufferNode_4 = ndarray::Array1::<f64>::zeros(0);
    let mut LpcNode_30_lpc_order = 10;
    let mut EnumerateIterNode_22 = (0..10).step_by(1).map(|x| x as f64).collect();
    let mut EnumerateIterNode_22_state = (0, 1, 10);
    loop {
        let out_37_0: ndarray::Array1<f64> = wav_file
            .try_recv()
            .unwrap()
            .into_iter()
            .map(|v| v as f64)
            .collect();
        let out_37_1 = sample_rate;
        let tmp_ExprNodes_18 = out_37_1;
        let out_18_0 = {
            let x = tmp_ExprNodes_18;
            x * 50 / 1000
        };
        let tmp_ExprNodes_3 = out_18_0;
        let out_3_0 = {
            let x = tmp_ExprNodes_3;
            round(x / 16) * 16
        };
        let tmp_ExprNodes_23 = out_22_0;
        let out_23_0 = {
            tmp_ExprNodes_23
                .iter()
                .map(|x| 0.5 * (1.0 - cos(2.0 * pi() * x / y)) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let tmp_ExprNodes_25 = out_23_0;
        let out_25_0 = {
            tmp_ExprNodes_25
                .into_iter()
                .map(|x| {
                    let (mut a, mut b) = (x * y, 0);
                    num_complex::Complex::new(a as f64, b as f64)
                })
                .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
        };
        if FFT_17_fft_size != out_25_0.len() {
            FFT_17_fft_size = out_25_0.len();
            FFT_17 = {
                let planner = rustfft::FftPlanner::new();
                let fft = planner.plan_fft_forward(FFT_17_fft_size);
                fft
            };
            FFT_17_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); FFT_17_fft_size];
        }
        let out_17_0 = {
            let mut out_17_0 = out_25_0.clone();
            FFT_17.process_with_scratch(out_17_0.as_mut_slice(), FFT_17_scratch_buf.as_mut_slice());
            out_17_0
        };
        let tmp_ExprNodes_27 = out_17_0;
        let out_27_0 = {
            tmp_ExprNodes_27
                .into_iter()
                .map(|num_complex::Complex { re: x, im: y }| {
                    let (mut a, mut b) = (x, y);
                    (a as f64, b as f64)
                })
                .collect::<ndarray::Array1<(f64, f64)>>()
        };
        let tmp_ExprNodes_2 = out_3_0;
        let out_2_0 = {
            let x = tmp_ExprNodes_2;
            round(x / 10)
        };
        let out_5_0 = STFTLayer_5
            .through_inner(out_4_0)
            .unwrap()
            .first()
            .unwrap()
            .to_owned();
        if STFTLayer_5_fft_size != 2400usize || STFTLayer_5_hop_size != out_2_0 {
            STFTLayer_5_fft_size = 2400usize;
            STFTLayer_5_hop_size = out_2_0;
            STFTLayer_5 = audio_analyzer_core::prelude::ToSpectrogramLayer::new(
                audio_analyzer_core::prelude::FftConfig {
                    fft_size: 2400usize,
                    hop_size: out_2_0,
                },
            );
        }
        let tmp_ExprNodes_7 = out_5_0;
        let out_7_0 = {
            tmp_ExprNodes_7
                .into_iter()
                .map(|num_complex::Complex { re: x, im: y }| 20.0 * log_10(abs(x)) as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let tmp_ExprNodes_0 = out_7_0;
        let out_0_0 = {
            tmp_ExprNodes_0
                .into_iter()
                .map(|x| {
                    let (mut a, mut b) = (x, 0);
                    num_complex::Complex::new(a as f64, b as f64)
                })
                .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
        };
        if IFFT_1_fft_size != out_0_0.len() {
            IFFT_1_fft_size = out_0_0.len();
            IFFT_1 = {
                let planner = rustfft::FftPlanner::new();
                let fft = planner.plan_fft_inverse(IFFT_1_fft_size);
                fft
            };
            IFFT_1_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); IFFT_1_fft_size];
        }
        let out_1_0 = {
            let mut out_1_0 = out_0_0.clone();
            IFFT_1.process_with_scratch(out_1_0.as_mut_slice(), IFFT_1_scratch_buf.as_mut_slice());
            out_1_0
        };
        let tmp_ExprNodes_10 = out_1_0;
        let out_10_0 = {
            tmp_ExprNodes_10
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
        let tmp_ExprNodes_15 = out_12_0;
        let out_15_0 = {
            tmp_ExprNodes_15
                .iter()
                .map(|x| x / 10000 as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let tmp_ExprNodes_26 = out_5_0;
        let out_26_0 = {
            tmp_ExprNodes_26
                .into_iter()
                .map(|num_complex::Complex { re: x, im: y }| {
                    let (mut a, mut b) = (x, y);
                    (a as f64, b as f64)
                })
                .collect::<ndarray::Array1<(f64, f64)>>()
        };
        let out_4_0 = {
            let extended = ndarray::concatenate(
                ndarray::Axis(0),
                &[CycleBufferNode_4.view(), out_37_0.view()],
            )
            .unwrap();
            let new_len = extended.len();
            if new_len > out_3_0 {
                let diff = new_len - out_3_0;
                let new_buffer = extended.slice(ndarray::s![diff..]);
                CycleBufferNode_4 = new_buffer.to_owned();
                assert!(new_buffer.len() == out_3_0);
            } else {
                CycleBufferNode_4 = extended.to_owned();
            }
            CycleBufferNode_4.clone()
        };
        let tmp_ExprNodes_36 = out_4_0;
        let out_36_0 = {
            tmp_ExprNodes_36
                .iter()
                .map(|x| (x > 0.0001) * x as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        if LpcNode_30_lpc_order != 100usize {
            LpcNode_30_lpc_order = 100usize;
        }
        let out_30_0 = linear_predictive_coding::calc_lpc_by_levinson_durbin(out_36_0, 100usize);
        let tmp_ExprNodes_34 = out_30_0;
        let out_34_0 = {
            tmp_ExprNodes_34
                .iter()
                .map(|x| x * 2 as f64)
                .collect::<ndarray::Array1<f64>>()
        };
        let out_22_0 = {
            if EnumerateIterNode_22_state != (0usize, 1usize, out_3_0) {
                EnumerateIterNode_22_state = (0usize, 1usize, out_3_0);
                EnumerateIterNode_22 = (0usize..out_3_0)
                    .step_by(1usize)
                    .map(|x| x as f64)
                    .collect();
            }
            EnumerateIterNode_22.clone()
        };
    }
}
