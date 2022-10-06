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
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table, TableState, Tabs},
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
    state: TableState,
}

impl App {
    pub fn new() -> Self {
        Self {
            index: 0,
            loader_percent: crate::IntWrap::new(0, 0..40),
            state: TableState::default(),
        }
    }

    pub fn next(&mut self) {
        let entries_len = ENTRIES.lock().len();

        if self.index < entries_len {
            self.index = (self.index + 1) % entries_len;
        } else {
            self.index = 0;
        }

        self.state.select(Some(self.index));
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = ENTRIES.lock().len();
        }
        self.state.select(Some(self.index));
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

    fn delete_entry(&mut self, index: usize) {
        let mut entries = ENTRIES.lock();
        let entry = entries.get_mut(index).cloned().unwrap();

        entries.get_mut(index).unwrap().deleting = Some(false);

        std::thread::spawn(move || {
            let p = entry.entry.path();
            let mut entries = ENTRIES.lock();

            match std::fs::remove_dir_all(p) {
                Ok(_) => {
                    entries.get_mut(index).unwrap().deleting = Some(true);
                }
                Err(_) => {
                    entries.get_mut(index).unwrap().deleting = None;
                }
            };
        });
    }

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(5)
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

            let list_entries = ENTRIES
                .lock()
                .iter()
                .map(|dir| {
                    debug!("Showing entry in tui: {}", dir.entry.path().display());
                    let mut size = bytesize::ByteSize(dir.size).to_string();

                    match dir.deleting {
                        Some(true) => size = "[DELETED]".to_owned(),
                        Some(false) => size.push_str(" [DELETING]"),
                        None => {}
                    }

                    Row::new([dir.entry.path().display().to_string(), size])
                })
                .collect::<Vec<_>>();

            let table = Table::new(list_entries)
                .style(Style::default().bg(Color::Black))
                .header(Row::new(["Path", "Size"]))
                .block(block)
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Min(10),
                ]);

            frame.render_stateful_widget(table, chunks[0], &mut self.state);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
