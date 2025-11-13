use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::Stylize,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use tokio::task::JoinHandle;

use crate::{
    comms::Comms,
    files::{DeletionState, DirEntry},
    sorting::{Column, Sorting},
};

pub fn pre_exit() -> anyhow::Result<()> {
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

impl From<usize> for Column {
    fn from(value: usize) -> Self {
        match value {
            0 => Column::name(),
            1 => Column::size(),
            _ => unreachable!(),
        }
    }
}

impl From<Column> for usize {
    fn from(value: Column) -> Self {
        if value.is_name() {
            0
        } else if value.is_size() {
            1
        } else {
            unreachable!()
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
    comms: Comms,
    sorting_state: Sorting,
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

    pub fn new(highlight_color: Color, comms: Comms) -> Self {
        Self {
            index: 0,
            state: TableState::default(),
            highlight_color,
            comms,
            sorting_state: Sorting::default(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn next(&mut self) {
        let entries_len = self.sorting_state.sorted().await.len();

        if self.index < entries_len.checked_sub(1).unwrap_or_default() {
            self.index += 1;
        } else {
            self.index = 0;
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn previous(&mut self) {
        let entries_len = self.sorting_state.sorted().await.len();

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
            let mut interval =
                tokio::time::interval(Duration::from_millis(1000 / 24) /* 24fps */);
            loop {
                // Ui-run
                debug!("Changed: {}", self.comms.changed());
                if self.comms.changed() {
                    self.comms.set_changed(false);
                    terminal.draw(|f| self.ui(f))?;
                }

                if event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Down => self.next().await,
                                KeyCode::Up => self.previous().await,
                                KeyCode::Tab | KeyCode::BackTab => {
                                    self.sorting_state.invert();
                                }
                                KeyCode::Right | KeyCode::Left => {
                                    self.sorting_state.switch_column();
                                }
                                KeyCode::Char(' ') => self.delete_entry(self.index).await,
                                code => {
                                    debug!("{}", code);
                                }
                            }
                            self.comms.set_changed(true);
                            self.comms.push_sort_tick().await?;
                        }
                    }
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
    async fn delete_entry(&mut self, index: usize) {
        let sorting_state = self.sorting_state.clone();
        let sorted = sorting_state.sorted_mut();

        // This must be a separate line to ensure that entries is not borrowed twice
        let entry_path = {
            let mut entries = sorted.lock().await;
            if entries
                .get_mut(index)
                .is_some_and(|entry| entry.state == DeletionState::Deleted)
            {
                entries.remove(index);

                return;
            }

            let entry = entries.get_mut(index).unwrap();
            entry.state = DeletionState::Deleting;

            entry.original.entry.path().to_path_buf()
        };

        let comms = self.comms.clone();
        tokio::spawn(async move {
            let state = if let Ok(()) = tokio::fs::remove_dir_all(entry_path).await {
                DeletionState::Deleted
            } else {
                DeletionState::Error
            };

            let mut entries = sorted.lock().await;
            let entry = entries.get_mut(index).unwrap();

            entry.state = state;
            comms.set_changed(true);
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
        let path_base = if self.sorting_state.column().is_name() {
            "> Path"
        } else {
            "Path"
        };

        let loading_text = if self.comms.loading() {
            " [LOADING]"
        } else {
            ""
        };

        format!("{path_base}{loading_text}")
    }

    fn size_header(&self) -> &'static str {
        if self.sorting_state.column().is_size() {
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
        let comms = self.comms.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(tick) = comms.pop_entry().await {
                    debug!("Popped entry. It contained data: {}", tick.is_some());
                    if let Some(entry) = tick {
                        sorting_state.add_entry(&entry).await;
                    }
                    sorting_state.sort().await;
                } else {
                    return;
                }
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
        let list_entries = self.sorting_state.blocking_sorted().clone();
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
