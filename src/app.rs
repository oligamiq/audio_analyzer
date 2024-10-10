use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Chart, Dataset},
};
use std::fmt::Debug;
use std::{
    any::Any,
    thread,
    time::{Duration, Instant},
};

use crate::{
    data::RawDataStreamLayer,
    layer::{
        layers::{AsAny, MultipleLayers, TailTrait},
        Layer,
    },
    trace_dbg,
};

//   Vec<f32>
// 音声ストリーム -> スペクトル -> メルスペクトル -> メルケプストラム

pub struct App<
    Input: 'static + Debug,
    Output: 'static + Debug,
    Tail: TailTrait<Input, Output> + 'static + Debug,
    NOutput: 'static + Debug,
> {
    data: Vec<(f64, f64)>,
    window: [f64; 2],
    layer: MultipleLayers<Input, Output, Tail, NOutput>,
}

impl<
        Input: 'static + Debug,
        Output: 'static + Debug,
        Tail: TailTrait<Input, Output> + 'static + Debug + AsAny,
        NOutput: 'static + Debug,
    > App<Input, Output, Tail, NOutput>
{
    pub fn new(layer: MultipleLayers<Input, Output, Tail, NOutput>) -> Self {
        Self {
            layer,
            window: [0.0, 20.0],
            data: vec![(0.0, 0.0)],
        }
    }

    pub fn run<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
        tick_rate: Duration,
    ) -> Result<()> {
        let mut last_tick = Instant::now();

        let length = self.layer.get_length();

        println!("length: {}", length);

        let to_mel_layer_ref = self
            .layer
            .get_nth::<crate::mel_layer::to_mel_layer::ToMelSpectrogramLayer>(1)
            .unwrap();

        let to_spec_layer_ref = self
            .layer
            .get_nth::<crate::mel_layer::fft_layer::ToSpectrogramLayer>(0)
            .unwrap();

        println!("{:?}", to_mel_layer_ref);
        println!("{:?}", to_spec_layer_ref);

        let layer = self.layer.get_0th_layer();
        println!("{:?}", layer);

        // {
        //     let mut tail = self.layer.get_tail();
        //     let layer = tail.__get_layer();
        //     println!("## {:?}", layer);
        //     let mut first_tail = Box::new(tail.__get_tail().unwrap() as &dyn Any);
        //     while let Some(__tail) = first_tail.__get_tail() {
        //         println!("{:?}", __tail);
        //         let any = __tail.as_any();
        //         if let Some(_) = any.downcast_ref::<()>() {
        //             break;
        //         } else {
        //             first_tail = __tail;
        //             if let Some(layer) = tail.__get_layer() {
        //                 println!("{:?}", layer);
        //             } else {
        //                 break;
        //             }
        //         }
        //     }
        // }

        // let t: <MultipleLayers<Input, Output, Tail, NOutput> as crate::layer::Layer>::InputType;

        self.layer.start();

        loop {
            terminal.draw(|frame| self.ui(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.modifiers == crossterm::event::KeyModifiers::CONTROL {
                        match key.code {
                            KeyCode::Char('c') => {
                                return Err(color_eyre::Report::msg("Terminated by Ctrl+C"))
                            }
                            _ => {}
                        }
                    }
                    // if key.code == KeyCode::Char('q') {
                    //     return Ok(());
                    // }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            // if let Ok(data) = receiver.try_recv() {
            //     trace_dbg!(data);
            // }
            // if let Ok(data) = vad_receiver.try_recv() {
            //     trace_dbg!(data);
            // }
        }
    }

    fn ui(&self, frame: &mut Frame) {
        let area = frame.size();

        let vertical = Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let horizontal = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]);
        let [chart1, bottom] = vertical.areas(area);
        let [line_chart, scatter] = horizontal.areas(bottom);

        self.render_chart1(frame, chart1);
        // render_line_chart(frame, line_chart);
        // render_scatter(frame, scatter);
    }

    fn render_chart1(&self, f: &mut Frame, area: Rect) {
        let x_labels = vec![
            Span::styled(
                format!("{}", self.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", (self.window[0] + self.window[1]) / 2.0)),
            Span::styled(
                format!("{}", self.window[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];
        let datasets = vec![
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.data),
            // Dataset::default()
            //     .name("data3")
            //     .marker(symbols::Marker::Braille)
            //     .style(Style::default().fg(Color::Yellow))
            //     .data(&self.data2),
        ];

        let chart = Chart::new(datasets)
            .block(Block::bordered().title("Chart 1".cyan().bold()))
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec!["-20".bold(), "0".into(), "20".bold()])
                    .bounds([-20.0, 20.0]),
            );

        f.render_widget(chart, area);
    }
}
