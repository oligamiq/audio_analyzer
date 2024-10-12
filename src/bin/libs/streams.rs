use audio_analyser::{
    data::RawDataStreamLayer,
    layer::{layers::MultipleLayers, Layer as _},
};
use log::debug;
use ndarray::{Array1, Axis as npAxis};
use std::fmt::Debug;
use std::{
    any::Any,
    thread,
    time::{Duration, Instant},
};

use crate::Result;

//   Vec<f32>
// 音声ストリーム -> スペクトル -> メルスペクトル -> メルケプストラム

pub struct Streamer {
    input_stream: Box<dyn RawDataStreamLayer>,
    layer: MultipleLayers,
    mel_psd_data: Vec<(f64, f64)>,
}

// impl Streamer {
//     pub fn new(input_stream: Box<dyn RawDataStreamLayer>, layer: MultipleLayers) -> Self {
//         Self {
//             input_stream,
//             layer,
//             mel_psd_data: vec![],
//         }
//     }

//     pub fn run(
//         mut self,
//         terminal: &mut Terminal<B>,
//         tick_rate: Duration,
//     ) -> Result<()> {
//         self.layer.check_types()?;

//         let length = self.layer.get_length();

//         debug!("length: {}", length);

//         let mut timeout = tick_rate.clone();

//         loop {
//             terminal.draw(|frame| self.ui(frame))?;

//             if crossterm::event::poll(timeout)? {
//                 if let Event::Key(key) = event::read()? {
//                     if key.modifiers == crossterm::event::KeyModifiers::CONTROL {
//                         match key.code {
//                             KeyCode::Char('c') => {
//                                 return Err(color_eyre::Report::msg("Terminated by Ctrl+C"))
//                             }
//                             _ => {}
//                         }
//                     }
//                     // if key.code == KeyCode::Char('q') {
//                     //     return Ok(());
//                     // }
//                 }
//             }

//             if let Some(data) = self.input_stream.try_recv() {
//                 // 0ms
//                 // debug!("{:?}", data);

//                 if let Ok(mel_data) = self.layer.through(&data as &dyn Any) {
//                     if mel_data.len() != 0 {
//                         debug!("{:?}", mel_data);
//                     }

//                     if let Some(mel_data) = mel_data
//                         .iter()
//                         .last()
//                         .map(|x| x.downcast_ref::<Array1<(f64, f64)>>().unwrap())
//                         .clone()
//                     {
//                         debug!("{:?}", mel_data);

//                         self.mel_psd_data = mel_data.to_vec();

//                         debug!("{:?}", self.mel_psd_data);
//                     }
//                     // debug!(mel_data);
//                 }

//                 // debug!(ave);
//                 timeout = Duration::from_millis(0);
//             } else {
//                 // 100ms
//                 timeout = tick_rate.clone();
//             }
//         }
//     }

//     fn ui(&self, frame: &mut Frame) {
//         let area = frame.area();

//         let vertical = Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)]);
//         let horizontal = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]);
//         let [chart1, bottom] = vertical.areas(area);
//         let [line_chart, scatter] = horizontal.areas(bottom);

//         self.render_chart1(frame, chart1);
//         // render_line_chart(frame, line_chart);
//         // render_scatter(frame, scatter);
//     }

//     fn render_chart1(&self, f: &mut Frame, area: Rect) {
//         // let x_labels = vec![
//         //     Span::styled(
//         //         format!("{}", self.window[0]),
//         //         Style::default().add_modifier(Modifier::BOLD),
//         //     ),
//         //     Span::raw(format!("{}", (self.window[0] + self.window[1]) / 2.0)),
//         //     Span::styled(
//         //         format!("{}", self.window[1]),
//         //         Style::default().add_modifier(Modifier::BOLD),
//         //     ),
//         // ];
//         let datasets = vec![
//             Dataset::default()
//                 .name("data2")
//                 .graph_type(GraphType::Line)
//                 .marker(symbols::Marker::Bar)
//                 .style(Style::default().fg(Color::Cyan))
//                 .data(&self.mel_psd_data),
//             // Dataset::default()
//             //     .name("data3")
//             //     .marker(symbols::Marker::Braille)
//             //     .style(Style::default().fg(Color::Yellow))
//             //     .data(&self.data2),
//         ];

//         let data_x_range_min = self
//             .mel_psd_data
//             .iter()
//             .map(|x| x.0)
//             .reduce(f64::min)
//             .unwrap_or_default();

//         let data_x_range_max = self
//             .mel_psd_data
//             .iter()
//             .map(|x| x.0)
//             .reduce(f64::max)
//             .unwrap_or_default();

//         let data_y_range_min = self
//             .mel_psd_data
//             .iter()
//             .map(|x| x.1)
//             .reduce(f64::min)
//             .unwrap_or_default();

//         let data_y_range_max = self
//             .mel_psd_data
//             .iter()
//             .map(|x| x.1)
//             .reduce(f64::max)
//             .unwrap_or_default();

//         let chart = Chart::new(datasets)
//             .block(Block::bordered().title("Chart".cyan().bold()))
//             .x_axis(
//                 Axis::default()
//                     .title("X Axis")
//                     .style(Style::default().fg(Color::Gray)) // .labels(x_labels)
//                     .bounds([data_x_range_min, data_x_range_max]),
//             )
//             .y_axis(
//                 Axis::default()
//                     .title("Y Axis")
//                     .style(Style::default().fg(Color::Gray))
//                     // .labels(vec!["-20".bold(), "0".into(), "20".bold()])
//                     .bounds([data_y_range_min, data_y_range_max]),
//             );

//         f.render_widget(chart, area);
//     }
// }
