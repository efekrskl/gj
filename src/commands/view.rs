use crate::AppContext;
use crate::database::Log;
use crate::utils::{logs_by_day_map, logs_to_list_items};
use anyhow::Result;
use clap::Args;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::crossterm::{cursor, event, execute};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use std::collections::BTreeMap;
use std::io::stdout;
use time::{Date, Duration, Month, OffsetDateTime};

#[derive(Args, Debug)]
pub struct ViewCommand {}

impl ViewCommand {
    pub fn execute(&self, ctx: &AppContext) -> Result<()> {
        let today = OffsetDateTime::now_local()
            .unwrap_or(OffsetDateTime::now_utc())
            .date();
        let logs = ctx.db.get_logs_by_year(today.year())?;

        run_calendar(logs, today, ctx)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Calendar,
    Entries,
}
struct CalendarState {
    cursor: Date,
    focus: Focus,
    entry_state: ListState,
    logs: BTreeMap<Date, Vec<Log>>,
}

impl CalendarState {
    pub fn new(logs: Vec<Log>, today: Date) -> CalendarState {
        CalendarState {
            cursor: today,
            focus: Focus::Calendar,
            entry_state: {
                let mut s = ListState::default();
                s.select(Some(0));
                s
            },
            logs: logs_by_day_map(logs).unwrap_or_default(),
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let [content_area, footer_area] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).areas(area);
        let [calendar_area, preview_area] =
            Layout::horizontal([Constraint::Length(96), Constraint::Min(20)]).areas(content_area);
        let [entry_list_area, entry_preview_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(11), Constraint::Min(5)])
            .areas(preview_area);

        self.draw_footer(frame, footer_area);
        self.draw_calendar(frame, calendar_area);
        self.draw_entry_list(frame, entry_list_area);
        self.draw_entry_preview(frame, entry_preview_area);
    }

    fn get_border_style(&self) -> Style {
        if self.focus == Focus::Entries {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }

    fn get_selected_log(&self) -> Option<&Log> {
        let logs = self.logs.get(&self.cursor)?;
        let selected_entry_idx = self.entry_state.selected()?;

        logs.get(selected_entry_idx)
    }
    fn draw_entry_preview(&self, frame: &mut Frame, area: Rect) {
        let list_border = self.get_border_style();
        let selected_entry_content = self
            .get_selected_log()
            .map(|entry| entry.content.as_str())
            .unwrap_or_default();

        frame.render_widget(
            Paragraph::new(selected_entry_content).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Entries ")
                    .border_style(list_border),
            ),
            area,
        );
    }

    fn draw_entry_list(&self, frame: &mut Frame, area: Rect) {
        let list_border = self.get_border_style();

        let empty_vec = vec![];
        let items: Vec<ListItem> =
            logs_to_list_items(self.logs.get(&self.cursor).unwrap_or(&empty_vec));

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Entries ")
                    .border_style(list_border),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.entry_state.clone());
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let status = match self.focus {
            Focus::Calendar => {
                "YEAR | ←→ day  ↑↓ week  PgUp/PgDn month  Tab entries  t today  q quit"
            }
            Focus::Entries => "YEAR | ↑↓ select entry  Tab calendar  q quit",
        };

        frame.render_widget(
            Paragraph::new(status).block(Block::default().borders(Borders::TOP)),
            area,
        );
    }

    fn draw_calendar(&self, frame: &mut Frame, area: Rect) {
        let mut store = CalendarEventStore::today(Style::default().bg(Color::DarkGray));

        for date in self.logs.keys() {
            store.add(*date, Style::default().fg(Color::Green))
        }

        store.add(
            self.cursor,
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12); 3])
            .split(area);

        let year = self.cursor.year();
        const COL_PER_ROW: usize = 4;
        for (row_idx, row_area) in rows.iter().enumerate() {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(24); COL_PER_ROW])
                .split(*row_area);

            for (col_idx, col_area) in cols.iter().enumerate() {
                let month_num = (row_idx * COL_PER_ROW) + col_idx + 1;
                let Ok(month) = Month::try_from(month_num as u8) else {
                    continue;
                };

                let first = Date::from_calendar_date(year, month, 1).unwrap();
                let is_cursor_month = self.cursor.to_calendar_date().1 == month;
                // todo: adjust colors
                let border = match (is_cursor_month, self.focus) {
                    (true, Focus::Calendar) => Style::default().fg(Color::Cyan),
                    (true, Focus::Entries) => Style::default().fg(Color::Gray),
                    (false, _) => Style::default().fg(Color::DarkGray),
                };

                let cal = Monthly::new(first, &store)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(format!(" {} {} ", month, year))
                            .border_style(border),
                    )
                    .show_weekdays_header(Style::default().fg(Color::DarkGray))
                    .show_surrounding(Style::default().fg(Color::DarkGray));

                frame.render_widget(cal, *col_area);
            }
        }
    }

    fn update_content_in_state(&mut self, new_content: String) {
        let Some(selected_idx) = self.entry_state.selected() else {
            return;
        };
        let Some(day_logs) = self.logs.get_mut(&self.cursor) else {
            return;
        };
        if let Some(log) = day_logs.get_mut(selected_idx) {
            log.content = new_content;
        }
    }
}

fn edit_log_content(original: &str) -> Result<Option<String>> {
    let edited = edit::edit(original)?;

    if original == edited {
        return Ok(None);
    }

    Ok(Some(edited))
}

fn suspend_tui() -> Result<()> {
    ratatui::restore();
    execute!(stdout(), cursor::Show)?;
    Ok(())
}

fn resume_tui() -> Result<ratatui::DefaultTerminal> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    Ok(terminal)
}

fn run_calendar(logs: Vec<Log>, today: Date, ctx: &AppContext) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut state = CalendarState::new(logs, today);

    loop {
        terminal.draw(|frame| state.draw(frame))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if state.focus == Focus::Calendar {
                let next = match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => state.cursor.checked_sub(Duration::days(1)),
                    KeyCode::Right => state.cursor.checked_add(Duration::days(1)),
                    KeyCode::Up => state.cursor.checked_sub(Duration::days(7)),
                    KeyCode::Down => state.cursor.checked_add(Duration::days(7)),
                    KeyCode::Char('t') => Some(today),
                    // todo: improve this by actually navigating a month
                    KeyCode::PageUp => state.cursor.checked_sub(Duration::days(31)),
                    KeyCode::PageDown => state.cursor.checked_add(Duration::days(31)),

                    KeyCode::Tab => {
                        state.focus = Focus::Entries;
                        None
                    }
                    _ => Some(state.cursor),
                };

                if let Some(date) = next {
                    state.cursor = date;
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab => state.focus = Focus::Calendar,
                    KeyCode::Up => {
                        state.entry_state.select_previous();
                        clamp_entry_selection(&mut state);
                    }
                    KeyCode::Down => {
                        state.entry_state.select_next();
                        clamp_entry_selection(&mut state);
                    }
                    KeyCode::Char('e') => {
                        let Some(selected_log) = state.get_selected_log() else {
                            continue;
                        };

                        suspend_tui()?;
                        let maybe_edited = edit_log_content(&selected_log.content)?;
                        if let Some(edited) = maybe_edited {
                            ctx.db.update_log(selected_log.id, &edited)?;
                            state.update_content_in_state(edited);
                        };
                        terminal = resume_tui()?;
                    }

                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn clamp_entry_selection(state: &mut CalendarState) {
    let len = state.logs.get(&state.cursor).unwrap_or(&vec![]).len();
    if len == 0 {
        state.entry_state.select(None);
        return;
    }
    let cur = state.entry_state.selected().unwrap_or(0);
    state.entry_state.select(Some(cur.min(len - 1)));
}
