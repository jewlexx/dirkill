use std::{io, path::PathBuf, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Tabs},
    Frame, Terminal,
};

pub struct App {
    files: Option<Vec<PathBuf>>,
    index: usize,
    loader_percent: dirlib::IntWrap<usize>,
}

impl App {
    pub const fn new() -> Self {
        Self {
            files: None,
            index: 0,
            loader_percent: dirlib::IntWrap::new(0, 0..100),
        }
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
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => self.next(),
                        KeyCode::Up => self.previous(),
                        _ => {}
                    }
                }
            }
        }

        crate::pre_exit()?;

        Ok(())
    }

    pub fn get_files(&self) -> Option<&Vec<PathBuf>> {
        self.files.as_ref()
    }

    pub fn set_files(&mut self, files: Vec<PathBuf>) {
        self.files = Some(files);
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
