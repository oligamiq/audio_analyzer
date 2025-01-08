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
    let mut lpc_node_30_lpc_order = 10;
    let mut buffer = Vec::new();
    let mut lpc_node_out_30_0s = Vec::new();
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
            let tmp_expr_nodes_36 = cycle_buffer_node_out_4_0.clone();
            let expr_nodes_out_36_0 = {
                tmp_expr_nodes_36
                    .iter()
                    .map(|&x| (float(x > 0.0001) * x) as f64)
                    .collect::<ndarray::Array1<f64>>()
            };
            if lpc_node_30_lpc_order != 100usize {
                lpc_node_30_lpc_order = 100usize;
            }
            let lpc_node_out_30_0 = linear_predictive_coding::calc_lpc_by_levinson_durbin(
                expr_nodes_out_36_0.view(),
                100usize,
            );
            lpc_node_out_30_0s.push(lpc_node_out_30_0.into_iter().map(
                |x| {
                    if x.is_nan() {
                        None
                    } else if x.is_infinite() {
                        None
                    } else {
                        Some(x)
                    }
                },
            ).collect::<Vec<_>>());
        }
    }

    return lpc_node_out_30_0s;
}
