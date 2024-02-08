use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    style::Stylize,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use parking_lot::{Mutex, MutexGuard};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use crate::files::{DeletionState, DirEntry};

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

#[derive(Debug)]
pub struct App {
    index: usize,
    state: TableState,
    highlight_color: Color,
    sorting: Sorting,
    sorting_inverted: bool,
}

impl App {
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
                            self.sorting_inverted = !self.sorting_inverted
                        }
                        KeyCode::Right => {
                            let old: usize = self.sorting.into();

                            self.sorting = if old >= 2 { 0 } else { old + 1 }.into();
                        }
                        KeyCode::Left => {
                            let old: usize = self.sorting.into();

                            self.sorting = if old == 0 { 2 } else { old - 1 }.into();
                        }
                        KeyCode::Enter => self.delete_entry(self.index),
                        _ => {}
                    }
                    *CHANGED.lock() = true;
                }
            }
        }

        pre_exit()?;

        Ok(())
    }

    #[tracing::instrument]
    fn delete_entry(&mut self, index: usize) {
        // This function calls 'entries.map' a lot.
        // This ensures that the passed closure gets access to the mutex guard,
        // While ensuring that said guard is dropped right after the closure.
        // This prevents the previous issue where the mutex was locked for this entire function,
        // Making the whole spawning of a thread, "[DELETING...]" status and updates pointless

        struct MutexMapper<'a, T>(&'a Mutex<T>);

        impl<'a, T> MutexMapper<'a, T> {
            /// Locks a mutex and allows a function to be called on said lock
            ///
            /// Is helpful when you need to lock a mutex, and have it unlock as soon as your statement is done
            pub fn map<R>(&self, func: impl Fn(MutexGuard<T>) -> R) -> R {
                let guard = self.0.lock();
                func(guard)
            }
        }

        std::thread::spawn(move || {
            let entries = MutexMapper(&ENTRIES);

            if entries.map(|mut guard| {
                // This must be a separate line to ensure that entries is not borrowed twice
                if guard.get_mut(index).unwrap().deletion_state == DeletionState::Deleted {
                    guard.remove(index);

                    true
                } else {
                    false
                }
            }) {
                return;
            }

            let entry_path = entries.map(|mut guard| {
                let entry = guard.get_mut(index).unwrap();
                entry.deletion_state = DeletionState::Deleting;

                entry.entry.path().to_path_buf()
            });

            match std::fs::remove_dir_all(entry_path) {
                Ok(_) => {
                    entries.map(|mut guard| {
                        let entry = guard.get_mut(index).unwrap();
                        entry.deletion_state = DeletionState::Deleted;
                    });
                }
                Err(_) => {
                    entries.map(|mut guard| {
                        let entry = guard.get_mut(index).unwrap();
                        entry.deletion_state = DeletionState::Deleted;
                    });
                }
            };
        });
    }

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        self.state.select(Some(self.index));

        let chunks = Layout::default()
            .constraints([
                Constraint::Percentage(5),
                Constraint::Percentage(5),
                Constraint::Percentage(90),
            ])
            .margin(5)
            .split(frame.size());

        let title = Span::styled(
            "DirKill",
            Style::default()
                .fg(self.highlight_color)
                .add_modifier(Modifier::ITALIC),
        );

        let me = Span::styled("Juliette Cordor", Style::default().fg(Color::LightGreen));

        let love = Span::styled("♥", Style::default().fg(Color::Red));

        let title = Paragraph::new(Spans::from(vec![
            title,
            Span::from(" was made with "),
            love,
            Span::from(" by "),
            me,
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(title, chunks[0]);

        let s = Span::styled(
            "Controls: <Left/Right> - Sort, <Tab> - Invert Sort, <Up/Down> - Navigate, <Enter> - Delete, <q> - Quit",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        let controls = Paragraph::new(s).alignment(Alignment::Center);

        frame.render_widget(controls, chunks[1]);

        let block = Block::default();

        let list_entries = {
            let mut unsorted_entries = ENTRIES.lock();

            unsorted_entries.sort_unstable_by(|old, entry| match self.sorting {
                Sorting::Name => old.entry.path().cmp(entry.entry.path()),
                // For some reason size sorting is inverted so we have to invert it back :)
                Sorting::Size => entry.size.cmp(&old.size),
            });

            if self.sorting_inverted {
                unsorted_entries.reverse();
            }

            unsorted_entries
                .iter()
                .map(|dir| {
                    (
                        dir.entry.path().display().to_string(),
                        (dir.size, dir.deletion_state),
                    )
                })
                .collect::<Vec<_>>()
        };

        let list_rows = list_entries
            .iter()
            .map(|entry| {
                Row::new([entry.0.clone(), {
                    let mut size = bytesize::ByteSize(entry.1 .0).to_string();

                    match entry.1 .1 {
                        DeletionState::Deleted => size = "[DELETED]".to_owned(),
                        DeletionState::Deleting => size.push_str(" [DELETING...]"),
                        DeletionState::Error => size = "[ERROR DELETING]".red().to_string(),
                        _ => {}
                    }

                    size
                }])
            })
            .collect::<Vec<_>>();

        let table = Table::new(list_rows)
            .style(Style::default().bg(Color::Black))
            .header(
                Row::new([
                    {
                        let path_base = if self.sorting == Sorting::Name {
                            "> Path"
                        } else {
                            "Path"
                        };

                        format!(
                            "{}{}",
                            path_base,
                            if *LOADING.lock() { " [LOADING]" } else { "" }
                        )
                    },
                    if self.sorting == Sorting::Size {
                        "> Size"
                    } else {
                        "Size"
                    }
                    .to_owned(),
                ])
                .style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(self.highlight_color)
                    .add_modifier(Modifier::BOLD),
            )
            .widths(&[
                Constraint::Percentage(50),
                Constraint::Length(30),
                Constraint::Min(10),
            ]);

        frame.render_stateful_widget(table, chunks[2], &mut self.state);
    }
}
