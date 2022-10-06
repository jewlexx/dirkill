use std::{io, path::PathBuf, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use parking_lot::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};

use crate::DirEntry;

pub fn pre_exit() -> anyhow::Result<()> {
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

pub static ENTRIES: Mutex<Vec<DirEntry>> = Mutex::new(Vec::new());
pub static LOADING: Mutex<bool> = Mutex::new(false);

pub struct App {
    index: usize,
    loader_percent: crate::IntWrap<usize>,
}

impl App {
    pub const fn new() -> Self {
        Self {
            index: 0,
            loader_percent: crate::IntWrap::new(0, 0..40),
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % ENTRIES.lock().len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = ENTRIES.lock().len();
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

        pre_exit()?;

        Ok(())
    }

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Min(0)].as_ref())
            .split(frame.size());

        if *LOADING.lock() {
            let mut text = "Loading Directory".to_owned();

            let dots = ".".repeat(*self.loader_percent.bump() / 10);
            text.push_str(&dots);

            let gauge = Paragraph::new(text);

            frame.render_widget(gauge, chunks[0]);
        } else {
            let block = Block::default()
                .title("Discovered Paths")
                .borders(Borders::ALL);

            let mut list_entries = ENTRIES
                .lock()
                .iter()
                .map(|dir| {
                    debug!("Showing entry in tui: {}", dir.entry.path().display());
                    Row::new([
                        // dir.entry.path().display().to_string(),
                        // bytesize::ByteSize(dir.size).to_string(),
                        "yo", "yay",
                    ])
                })
                .collect::<Vec<_>>();

            let table = Table::new(list_entries)
                .style(Style::default().bg(Color::Black))
                .header(Row::new(["Path", "Size"]))
                .block(block)
                .highlight_symbol(">>");

            frame.render_widget(table, chunks[0]);
        }
    }
}
