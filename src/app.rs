use std::{error::Error, io, path::PathBuf};

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
    widgets::{Block, Borders, Tabs},
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

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        match self.files {
            Some(ref files) => {}
            None => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [Constraint::Percentage(*self.loader_percent.get() as u16)].as_ref(),
                    )
                    .split(frame.size());

                let block = Block::default().borders(Borders::ALL).title("Loading");

                frame.render_widget(block, chunks[0]);

                self.loader_percent.bump();
            }
        };
    }
}
