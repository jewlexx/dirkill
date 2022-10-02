use std::{io, path::PathBuf, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use dirlib::DirEntry;
use parking_lot::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};

pub static FILES: Mutex<Option<Vec<DirEntry>>> = Mutex::new(None);

pub struct App {
    index: usize,
    loader_percent: dirlib::IntWrap<usize>,
}

impl App {
    pub const fn new() -> Self {
        Self {
            index: 0,
            loader_percent: dirlib::IntWrap::new(0, 0..40),
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % FILES.lock().as_ref().map(|x| x.len()).unwrap_or_default();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = FILES.lock().as_ref().map(|x| x.len()).unwrap_or_default();
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

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Min(0)].as_ref())
            .split(frame.size());

        match FILES.lock().as_ref() {
            Some(files_ref) => {
                let block = Block::default()
                    .title("Discovered Paths")
                    .borders(Borders::ALL);

                let mut list_entries = files_ref
                    .iter()
                    .map(|dir| {
                        Row::new([
                            dir.entry.path().to_string_lossy().to_string(),
                            bytesize::ByteSize(dir.size).to_string(),
                        ])
                    })
                    .collect::<Vec<_>>();

                list_entries.insert(0, Row::new(["Path", "Size"]));

                let table = Table::new(list_entries)
                    .style(Style::default().bg(Color::Black))
                    .block(block);

                frame.render_widget(table, chunks[0]);
            }
            None => {
                let mut text = "Loading Directory".to_owned();

                let dots = ".".repeat(*self.loader_percent.bump() / 10);
                text.push_str(&dots);

                let gauge = Paragraph::new(text);

                frame.render_widget(gauge, chunks[0]);
            }
        };
    }
}
