use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;

use crate::data::{test_data::TestDataType, RawDataType};

#[derive(Debug, Clone, Copy)]
pub struct App {
    data_from: RawDataType,
}

impl App {
    pub fn new() -> Self {
        Self {
            data_from: RawDataType::Test(TestDataType::TestData1),
        }
    }

    fn on_tick(&mut self) {}

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
                            KeyCode::Char('c') => return Err(color_eyre::Report::msg("Terminated by Ctrl+C")),
                            _ => {}
                        }
                    }
                    // if key.code == KeyCode::Char('q') {
                    //     return Ok(());
                    // }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
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

        // render_chart1(frame, chart1, app);
        // render_line_chart(frame, line_chart);
        // render_scatter(frame, scatter);
    }
}
