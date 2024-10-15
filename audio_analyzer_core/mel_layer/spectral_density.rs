// PSD layer

use std::{any::Any, fmt::Debug};

use log::debug;
use ndarray::{s, Array1, Array2, Axis};

use crate::layer::Layer;
use crate::Result;

#[derive(Debug, Clone)]
pub struct ToPowerSpectralDensityLayerConfig {
    pub sample_rate: f64,
    pub time_range: usize,
    pub n_mels: usize,
}

// https://gochikika.ntt.com/Visualization_and_EDA/spectral_visualization.html
// This use Welch's method.
// 今はピリオドグラムを使っているが、Welch's methodを使う
// STFTの結果を二乗して周波数毎に平均を取る
#[derive(Debug)]
pub struct ToPowerSpectralDensityLayer {
    config: ToPowerSpectralDensityLayerConfig,
    holder: Array2<f64>,
}

impl ToPowerSpectralDensityLayer {
    pub fn new(config: ToPowerSpectralDensityLayerConfig) -> Self {
        Self {
            config: config.clone(),
            holder: Array2::zeros((config.n_mels, 0)),
        }
    }
}

impl Layer for ToPowerSpectralDensityLayer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn through<'a>(&mut self, input: &'a dyn Any) -> Result<Vec<Box<(dyn Any + 'static)>>> {
        let Self { config, holder } = self;

        let data = input.downcast_ref::<Array2<f64>>().unwrap();

        // debug!("Data: {:?}", data);

        assert!(data.shape()[1] == 1);
        assert!(data.shape()[0] == config.n_mels);

        // dataから取り出す
        // let data = data.t()

        // holder.assign_elem(data);

        assert!(holder.shape()[0] == config.n_mels);

        *holder = ndarray::concatenate(Axis(1), &[holder.view(), data.view()]).unwrap();

        // holder.axis_iter_mut(Axis(1)).for_each(|mut x| {
        //     debug!("$$ X 1: {:?}", x);

        //     debug!("$$ X 2: {:?}", x);
        // });

        // debug!("$$ Holder: {:?}", holder.shape());

        // holder = ndarray::arr2(&[holder.to_owned(), data.t().to_owned()])
        //     .concatenate(Axis(0));

        if holder.shape()[1] < config.time_range {
            return Ok(vec![]);
        }

        // remove first element if holder is too long

        if holder.shape()[1] > config.time_range {
            *holder = holder.slice(s![.., 1..]).to_owned();
        }

        // debug!("Holder: {:?}", holder);

        assert!(holder.shape()[1] == config.time_range);
        assert!(holder.shape()[0] == config.n_mels);

        // use holder to calculate PSD
        let psd: Array1<(f64, f64)> = holder
            .axis_iter(Axis(0))
            .enumerate()
            .map(|(i, x)| {
                let sum = x.mapv(|x| x.powi(2)).sum() / config.time_range as f64;
                let freq = i as f64 * config.sample_rate / config.n_mels as f64;
                (freq, sum)
            })
            .collect::<Array1<_>>();

        debug!("PSD: {:?}", psd);

        Ok(vec![Box::new(psd) as Box<dyn Any>])
    }

    fn input_type(&self) -> &'static str {
        "Array2<f64>"
    }

    fn output_type(&self) -> &'static str {
        "Array1<(f64, f64)>"
    }
}
