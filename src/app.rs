use std::{io, time::Duration};

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
};

pub fn pre_exit() -> anyhow::Result<()> {
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

// Some state handlers
pub static ENTRIES: Mutex<Vec<DirEntry>> = Mutex::new(Vec::new());
pub static LOADING: Mutex<bool> = Mutex::new(true);
pub static CHANGED: Mutex<bool> = Mutex::new(false);

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sorting {
    #[default]
    Name,
    Size,
}

impl From<usize> for Sorting {
    fn from(value: usize) -> Self {
        match value {
            0 => Sorting::Name,
            1 => Sorting::Size,
            _ => unreachable!(),
        }
    }
}

impl From<Sorting> for usize {
    fn from(value: Sorting) -> Self {
        match value {
            Sorting::Name => 0,
            Sorting::Size => 1,
        }
    }
}

struct Entry {
    path: String,
    size: u64,
    state: DeletionState,
}

impl<'a> From<&'a DirEntry> for Entry {
    fn from(entry: &'a DirEntry) -> Self {
        Self {
            path: entry.entry.path().display().to_string(),
            size: entry.size,
            state: entry.deletion_state,
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
    sorting: Sorting,
    sorting_inverted: bool,
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

    pub fn new(highlight_color: Color) -> Self {
        Self {
            index: 0,
            state: TableState::default(),
            highlight_color,
            sorting: Sorting::default(),
            sorting_inverted: false,
        }
    }

    #[tracing::instrument]
    pub fn next(&mut self) {
        let entries_len = ENTRIES.lock().len();

        if self.index < entries_len - 1 {
            self.index += 1;
        } else {
            self.index = 0;
        }
    }

    #[tracing::instrument]
    pub fn previous(&mut self) {
        let entries_len = ENTRIES.lock().len();

        if self.index > 0 {
            self.index = self.index.wrapping_sub(1);
        } else {
            self.index = entries_len.wrapping_sub(1);
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            if *CHANGED.lock() {
                terminal.draw(|f| self.ui(f))?;
            }

            if event::poll(Duration::ZERO)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => self.next(),
                        KeyCode::Up => self.previous(),
                        KeyCode::Tab | KeyCode::BackTab => {
                            self.sorting_inverted = !self.sorting_inverted;
                        }
                        KeyCode::Right => {
                            let old: usize = self.sorting.into();

                            self.sorting = if old >= 2 { 0 } else { old + 1 }.into();
                        }
                        KeyCode::Left => {
                            let old: usize = self.sorting.into();

                            self.sorting = if old == 0 { 2 } else { old - 1 }.into();
                        }
                        KeyCode::Char(' ') => self.delete_entry(self.index),
                        _ => {}
                    }
                    *CHANGED.lock() = true;
                }
                assert!(!CHANGED.is_locked());
            }
        }

        pre_exit()?;

        Ok(())
    }

    #[tracing::instrument]
    fn delete_entry(&mut self, index: usize) {
        std::thread::spawn(move || {
            if ENTRIES.map(|mut entries| {
                // This must be a separate line to ensure that entries is not borrowed twice
                if entries
                    .get_mut(index)
                    .is_some_and(|entry| entry.deletion_state == DeletionState::Deleted)
                {
                    entries.remove(index);

                    true
                } else {
                    false
                }
            }) {
                return;
            }
            assert!(!ENTRIES.is_locked());

            let entry_path = ENTRIES.map(|mut guard| {
                let entry = guard.get_mut(index).unwrap();
                entry.deletion_state = DeletionState::Deleting;

                entry.entry.path().to_path_buf()
            });
            assert!(!ENTRIES.is_locked());

            if let Ok(()) = std::fs::remove_dir_all(entry_path) {
                ENTRIES.map(|mut guard| {
                    let entry = guard.get_mut(index).unwrap();
                    entry.deletion_state = DeletionState::Deleted;
                });
                assert!(!ENTRIES.is_locked());
            } else {
                ENTRIES.map(|mut guard| {
                    let entry = guard.get_mut(index).unwrap();
                    entry.deletion_state = DeletionState::Error;
                });
                assert!(!ENTRIES.is_locked());
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
        let path_base = if self.sorting == Sorting::Name {
            "> Path"
        } else {
            "Path"
        };

        let loading_text = if *LOADING.lock() { " [LOADING]" } else { "" };
        assert!(!LOADING.is_locked());

        format!("{path_base}{loading_text}")
    }

    fn size_header(&self) -> &'static str {
        if self.sorting == Sorting::Size {
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

    fn ui(&mut self, frame: &mut Frame<'_>) {
        self.state.select(Some(self.index));

        let chunks = Layout::default()
            .constraints(Self::LAYOUT_CONSTRAINTS)
            .margin(5)
            .split(frame.area());

        frame.render_widget(self.title(), chunks[0]);

        frame.render_widget(Self::controls(), chunks[1]);

        let block = Block::default();

        let list_entries = {
            // TODO: Sort on a separate thread
            let mut unsorted_entries = ENTRIES.lock();

            unsorted_entries.sort_unstable_by(|a, b| match self.sorting {
                Sorting::Name => a.entry.path().cmp(b.entry.path()),
                // Sorting is inverse here, because we want the larger size to be first
                Sorting::Size => b.size.cmp(&a.size),
            });

            if self.sorting_inverted {
                unsorted_entries.reverse();
            }

            // Lock dropped here
            unsorted_entries.iter().map(Entry::from).collect::<Vec<_>>()
        };

        // Check that the lock was dropped at the end of the above scope
        assert!(!ENTRIES.is_locked());

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
