use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use parking_lot::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use crate::files::DirEntry;

pub fn pre_exit() -> anyhow::Result<()> {
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

pub static ENTRIES: Mutex<Vec<DirEntry>> = Mutex::new(Vec::new());
pub static LOADING: Mutex<bool> = Mutex::new(true);

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Sorting {
    #[default]
    None,

    Name,
    Size,
}

impl From<usize> for Sorting {
    fn from(value: usize) -> Self {
        match value {
            1 => Sorting::Name,
            2 => Sorting::Size,
            _ => Sorting::None,
        }
    }
}

impl From<Sorting> for usize {
    fn from(value: Sorting) -> Self {
        match value {
            Sorting::Name => 1,
            Sorting::Size => 2,
            Sorting::None => 0,
        }
    }
}

pub struct App {
    index: usize,
    state: TableState,
    highlight_color: Color,
    sorting: Sorting,
}

impl App {
    pub fn new(highlight_color: Color) -> Self {
        Self {
            index: 0,
            state: TableState::default(),
            highlight_color,
            sorting: Sorting::default(),
        }
    }

    pub fn next(&mut self) {
        let entries_len = ENTRIES.lock().len();

        if self.index < entries_len - 1 {
            self.index += 1;
        } else {
            self.index = 0;
        }

        self.state.select(Some(self.index));
    }

    pub fn previous(&mut self) {
        let entries_len = ENTRIES.lock().len();

        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = entries_len - 1;
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
            "Controls: <Left/Right> - Sort, <Up/Down> - Navigate, <Enter> - Delete, <q> - Quit",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        let controls = Paragraph::new(s).alignment(Alignment::Center);

        frame.render_widget(controls, chunks[1]);

        let block = Block::default();

        let mut unsorted_entries = ENTRIES.lock();

        unsorted_entries.sort_by(|old, entry| match self.sorting {
            Sorting::Name => old.entry.path().cmp(entry.entry.path()),
            // For some reason size sorting is inverted so we have to invert it back :)
            Sorting::Size => entry.size.cmp(&old.size),
            Sorting::None => std::cmp::Ordering::Equal,
        });

        let list_entries = ENTRIES
            .lock()
            .iter()
            .map(|dir| {
                (
                    dir.entry.path().display().to_string(),
                    (dir.size, dir.deleting),
                )
            })
            .collect::<Vec<_>>();

        let list_rows = list_entries
            .iter()
            .map(|entry| {
                Row::new([entry.0.clone(), {
                    let mut size = bytesize::ByteSize(entry.1 .0).to_string();

                    match entry.1 .1 {
                        Some(true) => size = "[DELETED]".to_owned(),
                        Some(false) => size.push_str(" [DELETING...]"),
                        None => {}
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
