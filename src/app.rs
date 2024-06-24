use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Chart, Dataset},
};

use crate::{data::RawDataLayer, stream::MelLayer};

#[derive(Clone)]
pub struct App<R: RawDataLayer, T: MelLayer> {
    data: Vec<(f64, f64)>,
    window: [f64; 2],
    mel_layer: T,
    raw_data_layer: R,
}

impl<R, T> App<R, T>
where
    R: RawDataLayer,
    T: MelLayer,
{
    pub fn new(raw_data_layer: R, mel_layer: T) -> Self {
        Self {
            raw_data_layer,
            mel_layer,
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
