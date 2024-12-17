use std::{
    io,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::Receiver;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    style::Stylize,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use parking_lot::Mutex;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use crate::{
    files::{DeletionState, DirEntry},
    locks::LockMap,
    sorting::{Column, Sorting},
};

pub fn pre_exit() -> anyhow::Result<()> {
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

// Some state handlers
pub static LOADING: Mutex<bool> = Mutex::new(true);
pub static CHANGED: Mutex<bool> = Mutex::new(false);

impl From<usize> for Column {
    fn from(value: usize) -> Self {
        match value {
            0 => Column::Name,
            1 => Column::Size,
            _ => unreachable!(),
        }
    }
}

impl From<Column> for usize {
    fn from(value: Column) -> Self {
        match value {
            Column::Name => 0,
            Column::Size => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: String,
    pub size: u64,
    pub state: DeletionState,
    pub original: DirEntry,
}

impl<'a> From<&'a DirEntry> for Entry {
    fn from(entry: &'a DirEntry) -> Self {
        Self {
            path: entry.entry.path().display().to_string(),
            size: entry.size,
            state: entry.deletion_state,
            original: entry.clone(),
        }
    }
}

impl Entry {
    fn to_row(&self) -> Row<'_> {
        Row::new([self.path.clone(), {
            match self.state {
                DeletionState::Deleted => "[DELETED]".to_string(),
                DeletionState::Error => "[ERROR DELETING]".red().to_string(),
                state => {
                    let mut size = bytesize::ByteSize(self.size).to_string();

                    if matches!(state, DeletionState::Deleting) {
                        size.push_str(" [DELETING...]");
                    }

                    size
                }
            }
        }])
    }
}

#[derive(Debug)]
pub struct App {
    index: usize,
    state: TableState,
    highlight_color: Color,
    rx: Receiver<DirEntry>,
    sorting_state: Arc<Mutex<Sorting>>,
}

impl App {
    const TABLE_CONSTRAINTS: &[Constraint] = &[
        Constraint::Percentage(50),
        Constraint::Length(30),
        Constraint::Min(10),
    ];

    const LAYOUT_CONSTRAINTS: &[Constraint] = &[
        Constraint::Percentage(5),
        Constraint::Percentage(5),
        Constraint::Percentage(90),
    ];

    pub fn new(highlight_color: Color, rx: Receiver<DirEntry>) -> Self {
        Self {
            index: 0,
            state: TableState::default(),
            highlight_color,
            rx,
            sorting_state: Arc::new(Mutex::new(Sorting::default())),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn next(&mut self) {
        let entries_len = self.sorting_state.lock().sorted().len();

        if self.index < entries_len - 1 {
            self.index += 1;
        } else {
            self.index = 0;
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn previous(&mut self) {
        let entries_len = self.sorting_state.lock().sorted().len();

        if self.index > 0 {
            self.index = self.index.wrapping_sub(1);
        } else {
            self.index = entries_len.wrapping_sub(1);
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / 30.0));
            loop {
                // Ui-run
                // Scope exists so that `CHANGED` is dropped before the next interval tick
                {
                    let mut has_changed = CHANGED.lock();
                    if *has_changed {
                        terminal.draw(|f| self.ui(f))?;
                        *has_changed = false;
                    }
                };

                if event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Down => self.next(),
                            KeyCode::Up => self.previous(),
                            KeyCode::Tab | KeyCode::BackTab => self.sorting_state.lock().invert(),
                            KeyCode::Right | KeyCode::Left => {
                                self.sorting_state.lock().switch_column();
                            }
                            KeyCode::Char(' ') => self.delete_entry(self.index),
                            code => {
                                debug!("{}", code);
                            }
                        }
                        *CHANGED.lock() = true;
                    }
                    assert!(!CHANGED.is_locked());
                }

                interval.tick().await;
            }

            anyhow::Ok(())
        })
        .await??;

        pre_exit()?;

        Ok(())
    }

    #[tracing::instrument]
    fn delete_entry(&mut self, index: usize) {
        let sorting_state = self.sorting_state.clone();
        std::thread::spawn(move || {
            if sorting_state.map(|mut state| {
                // This must be a separate line to ensure that entries is not borrowed twice
                let entries = state.sorted_mut();
                if entries
                    .get_mut(index)
                    .is_some_and(|entry| entry.state == DeletionState::Deleted)
                {
                    entries.remove(index);

                    true
                } else {
                    false
                }
            }) {
                return;
            }

            let entry_path = sorting_state.map(|mut guard| {
                let entry = guard.sorted_mut().get_mut(index).unwrap();
                entry.state = DeletionState::Deleting;

                entry.original.entry.path().to_path_buf()
            });

            if let Ok(()) = std::fs::remove_dir_all(entry_path) {
                sorting_state.map(|mut guard| {
                    let entry = guard.sorted_mut().get_mut(index).unwrap();
                    entry.state = DeletionState::Deleted;
                });
            } else {
                sorting_state.map(|mut guard| {
                    let entry = guard.sorted_mut().get_mut(index).unwrap();
                    entry.state = DeletionState::Error;
                });
            };
        });
    }

    fn title(&self) -> Paragraph<'_> {
        let title = Span::styled(
            "DirKill",
            Style::default()
                .fg(self.highlight_color)
                .add_modifier(Modifier::ITALIC),
        );

        let me = Span::styled("Juliette Cordor", Style::default().fg(Color::LightGreen));

        let love = Span::styled("♥", Style::default().fg(Color::Red));

        Paragraph::new(Line::from(vec![
            title,
            Span::from(" was made with "),
            love,
            Span::from(" by "),
            me,
        ]))
        .alignment(Alignment::Center)
    }

    fn controls<'a>() -> Paragraph<'a> {
        let controls = Span::styled(
            "Controls: <Left/Right> - Sort, <Tab> - Invert Sort, <Up/Down> - Navigate, <Space> - Delete, <q> - Quit",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        Paragraph::new(controls).alignment(Alignment::Center)
    }

    fn path_header(&self) -> String {
        let path_base = if self.sorting_state.lock().column() == Column::Name {
            "> Path"
        } else {
            "Path"
        };

        let loading_text = if *LOADING.lock() { " [LOADING]" } else { "" };
        assert!(!LOADING.is_locked());

        format!("{path_base}{loading_text}")
    }

    fn size_header(&self) -> &'static str {
        if self.sorting_state.lock().column() == Column::Size {
            "> Size"
        } else {
            "Size"
        }
    }

    fn table_headers<'r>(&self) -> Row<'r> {
        Row::new([self.path_header(), self.size_header().to_owned()])
            .style(Style::default().add_modifier(Modifier::BOLD))
    }

    fn row_highlight(&self) -> Style {
        Style::default()
            .bg(self.highlight_color)
            .add_modifier(Modifier::BOLD)
    }

    pub fn sort_entries(&self) -> JoinHandle<()> {
        let sorting_state = self.sorting_state.clone();
        let rx = self.rx.clone();

        thread::spawn(move || loop {
            if let Ok(entry) = rx.recv() {
                let mut sorting_state = sorting_state.lock();
                sorting_state.add_entry(&entry);
                sorting_state.sort();
            } else {
                return;
            }
        })
    }

    #[tracing::instrument(skip_all)]
    fn ui(&mut self, frame: &mut Frame<'_>) {
        self.state.select(Some(self.index));

        let chunks = Layout::default()
            .constraints(Self::LAYOUT_CONSTRAINTS)
            .margin(5)
            .split(frame.area());

        frame.render_widget(self.title(), chunks[0]);

        frame.render_widget(Self::controls(), chunks[1]);

        let block = Block::default();

        trace!("Starting sort");
        let list_entries = self.sorting_state.lock().sorted().to_vec();
        trace!("Finished sort");

        let list_rows = list_entries
            .iter()
            .map(|entry| entry.to_row())
            .collect::<Vec<_>>();

        let table = Table::new(list_rows, Self::TABLE_CONSTRAINTS)
            .style(Style::default().bg(Color::Black))
            .header(self.table_headers())
            .block(block)
            .row_highlight_style(self.row_highlight());

        frame.render_stateful_widget(table, chunks[2], &mut self.state);
    }
}
