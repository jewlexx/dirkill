use std::{error::Error, io, path::PathBuf, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Tabs},
    Frame, Terminal,
};

#[derive(Default)]
pub struct App {
    pub files: Option<Vec<PathBuf>>,
    pub index: usize,
    pub loader_percent: dirlib::IntWrap<usize>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.files.as_ref().map(|x| x.len()).unwrap_or_default();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.files.as_ref().map(|x| x.len()).unwrap_or_default();
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => self.next(),
                        KeyCode::Up => self.previous(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        match self.files {
            Some(ref files) => {
                println!("Is Some");
            }
            None => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(5)
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(frame.size());

                let gauge = Gauge::default()
                    .gauge_style(Style::default().fg(Color::Green))
                    .percent(*self.loader_percent.bump() as u16);

                frame.render_widget(gauge, chunks[0]);
            }
        };
    }
}
