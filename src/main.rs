use core::fmt;
use crossterm::event::{
    self,
    KeyCode::{self},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect, Spacing},
    style::{Color, Style, Stylize},
    symbols::{self, merge::MergeStrategy},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, Padding, Paragraph, Row, Table, TableState, Tabs},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self},
    path::Path,
    str::FromStr,
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
struct Period {
    first_time: String,
    second_time: String,
}

impl Period {
    fn default() -> Self {
        Self {
            first_time: "00:00".to_string(),
            second_time: "00:00".to_string(),
        }
    }

    fn to_line_string(&self) -> String {
        format!("{}-\n{}", self.first_time, self.second_time)
    }

    fn to_text(&self) -> Text<'static> {
        let first_time = if self.first_time == "00:00" {
            self.first_time.clone().gray().dim()
        } else {
            self.first_time.clone().into()
        };

        let second_time = if self.second_time == "00:00" {
            self.second_time.clone().gray().dim()
        } else {
            self.second_time.clone().into()
        };

        Text::from(Line::from(vec![first_time, Span::raw("-"), second_time]))
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.first_time, self.second_time)
    }
}

impl FromStr for Period {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').map(|p| p.trim()).collect();

        if parts.len() != 2 {
            return Err(format!("expected 2 fields, got {}", parts.len()));
        }

        Ok(Period {
            first_time: parts[0].to_string(),
            second_time: parts[1].to_string(),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Subject {
    name: String,
    room: String,
    teacher: String,
}

impl Subject {
    fn default() -> Self {
        Self {
            name: String::from("None"),
            room: String::from("None"),
            teacher: String::from("None"),
        }
    }

    fn to_text(&self) -> Text<'static> {
        let name_line = if self.name == "None" {
            Line::from(self.name.clone().gray().dim())
        } else {
            Line::from(self.name.clone())
        };

        let room_line = if self.room == "None" {
            Line::from(self.room.clone().gray().dim())
        } else {
            Line::from(self.room.clone())
        };

        Text::from(vec![name_line, room_line])
    }
}

impl FromStr for Subject {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("\n").map(|p| p.trim()).collect();

        if parts.len() != 2 {
            return Err(format!("expected 2 fields, got {}", parts.len()));
        }

        Ok(Subject {
            name: parts[0].to_string(),
            room: parts[1].to_string(),
            teacher: "None".to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    times: Vec<(u8, Period)>,
    content: Vec<(u8, Column, Subject)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Column {
    Time,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

#[allow(dead_code)]
struct App {
    selected_tab: usize,
    save_file: String,
    times: Vec<(u8, Period)>,
    state_table: TableState,
    state_settings: TableState,
    edit_mode: bool,
    edit_buf: String,
    content: Vec<(u8, Column, Subject)>,
    create_popup: bool,
    selected_create_tab: usize,
    state_create_table: TableState,
    temp_subject: Subject,
    index_subjects: Vec<String>,
    index_rooms: Vec<String>,
    index_teachers: Vec<String>,
    color: String,
}

const COLUMNS: [Column; 6] = [
    Column::Time,
    Column::Monday,
    Column::Tuesday,
    Column::Wednesday,
    Column::Thursday,
    Column::Friday,
];

impl App {
    fn new() -> Self {
        let content: Vec<(u8, Column, Subject)> = COLUMNS
            .iter()
            .flat_map(|&column| (1..=11).map(move |n| (n, column, Subject::default())))
            .collect();
        let times: Vec<(u8, Period)> = (1..=11).map(|n| (n, Period::default())).collect();
        Self {
            selected_tab: 0,
            save_file: "save-file.txt".to_string(),
            times,
            state_table: TableState::default()
                .with_selected(0)
                .with_selected_column(1),
            state_settings: TableState::default()
                .with_selected(0)
                .with_selected_column(1),
            edit_mode: false,
            edit_buf: String::new(),
            content,
            create_popup: false,
            selected_create_tab: 0,
            state_create_table: TableState::default()
                .with_selected(0)
                .with_selected_column(1),
            temp_subject: Subject::default(),
            index_subjects: vec![],
            index_rooms: vec![],
            index_teachers: vec![],
            color: String::from("#00ff00"),
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if !Path::new(&self.save_file).exists() {
            self.save_to_file()?;
        } else {
            let json = fs::read_to_string(&self.save_file)?;
            let save_data: SaveData = serde_json::from_str(&json)?;
            self.times = save_data.times;
            self.content = save_data.content;
        }

        loop {
            terminal.draw(|frame| self.render(frame, self.selected_tab))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    _ if !self.create_popup => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc if !self.edit_mode => return Ok(()),
                        KeyCode::Tab if !self.edit_mode => {
                            self.selected_tab = (self.selected_tab + 1) % 3
                        }
                        _ => match self.selected_tab {
                            0 => match key.code {
                                KeyCode::Enter => {
                                    if let (Some(row_idx), Some(col_idx)) = (
                                        self.state_table.selected(),
                                        self.state_table.selected_column(),
                                    ) {
                                        self.temp_subject = self
                                            .get_selected_subject(row_idx, col_idx)
                                            .cloned()
                                            .unwrap_or(Subject::default());
                                    }
                                    self.create_popup = !self.create_popup;
                                    if self.temp_subject.name == "None" {
                                        self.edit_mode = true;
                                    }
                                }
                                KeyCode::Left => {
                                    self.state_table.select_previous_column();
                                    if self.state_table.selected_column() == Some(0) {
                                        self.state_table.select_next_column();
                                    }
                                }
                                KeyCode::Right => {
                                    self.state_table.select_next_column();
                                    if self.state_table.selected_column() == Some(0) {
                                        self.state_table.select_previous_column();
                                    }
                                }
                                KeyCode::Down => {
                                    self.state_table.select_next();
                                }
                                KeyCode::Up => {
                                    self.state_table.select_previous();
                                }
                                KeyCode::Char('c') => {
                                    if let Some((row_idx, col_idx)) = self
                                        .state_table
                                        .selected()
                                        .zip(self.state_table.selected_column())
                                    {
                                        self.set_selected_subject(
                                            row_idx,
                                            col_idx,
                                            Subject::default(),
                                        );
                                    }
                                    self.save_to_file()?;
                                }
                                _ => {}
                            },
                            1 => match key.code {
                                _ if self.edit_mode => match key.code {
                                    KeyCode::Enter => {
                                        if let Some(i) = self.state_settings.selected() {
                                            self.times[i].1 =
                                                self.edit_buf.parse().unwrap_or(Period::default());
                                            if self.times[i].1.first_time.is_empty() {
                                                self.times[i].1.first_time = "00:00".to_string();
                                            } else if self.times[i].1.second_time.is_empty()
                                                || self.times[i].1.second_time.len() < 5
                                            {
                                                self.times[i].1.second_time = "00:00".to_string();
                                            }
                                        }
                                        if self.state_settings.selected() != Some(10) {
                                            self.state_settings.select_next();
                                            self.edit_buf.clear();
                                        } else {
                                            self.edit_mode = false;
                                            self.edit_buf.clear();
                                        }
                                        self.save_to_file()?;
                                    }
                                    KeyCode::Esc => {
                                        self.edit_mode = false;
                                        self.edit_buf.clear();
                                    }
                                    KeyCode::Char(c)
                                        if c.is_ascii_digit() || matches!(c, '-' | ':') =>
                                    {
                                        let digit_count = self
                                            .edit_buf
                                            .chars()
                                            .filter(|c| c.is_ascii_digit())
                                            .count();

                                        let valid = match digit_count {
                                            0 | 4 => matches!(c, '0'..='2'),
                                            2 | 6 => matches!(c, '0'..='5'),
                                            8.. => false,
                                            _ => true,
                                        };

                                        if valid {
                                            self.edit_buf.push(c);
                                            match digit_count {
                                                1 | 5 if !self.edit_buf.ends_with(':') => {
                                                    self.edit_buf.push(':')
                                                }
                                                3 if !self.edit_buf.ends_with('-') => {
                                                    self.edit_buf.push('-')
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        if matches!(
                                            self.edit_buf.chars().last(),
                                            Some(':') | Some('-')
                                        ) {
                                            self.edit_buf.pop();
                                        }
                                        self.edit_buf.pop();
                                    }
                                    _ => {}
                                },
                                KeyCode::Enter => {
                                    if let Some(i) = self.state_settings.selected() {
                                        let current = self.times[i].1.clone();
                                        self.edit_buf = if current == Period::default() {
                                            String::new()
                                        } else {
                                            current.to_string()
                                        };
                                        let last_time: String = self
                                            .edit_buf
                                            .chars()
                                            .skip(self.edit_buf.chars().count().saturating_sub(5))
                                            .collect();
                                        if last_time == "00:00" {
                                            for _ in 0..5 {
                                                self.edit_buf.pop();
                                            }
                                        }
                                        self.edit_mode = true;
                                    }
                                }
                                // KeyCode::Left => self.state_settings.select_previous_column(),
                                // KeyCode::Right => self.state_settings.select_next_column(),
                                KeyCode::Down => self.state_settings.select_next(),
                                KeyCode::Up => self.state_settings.select_previous(),
                                _ => {}
                            },
                            2 => {}
                            _ => {}
                        },
                    },
                    _ if self.create_popup => match key.code {
                        _ if self.edit_mode => match key.code {
                            KeyCode::Enter => {
                                match self.state_create_table.selected() {
                                    Some(0) => {
                                        if self.edit_buf.is_empty() {
                                            self.temp_subject.name = "None".to_string();
                                        } else {
                                            self.temp_subject.name = self.edit_buf.clone();
                                        }
                                        self.state_create_table.select_next();
                                        self.edit_mode = self.temp_subject.room == "None";
                                    }
                                    Some(1) => {
                                        if self.edit_buf.is_empty() {
                                            self.temp_subject.room = "None".to_string();
                                        } else {
                                            self.temp_subject.room = self.edit_buf.clone();
                                        }
                                        self.state_create_table.select_next();
                                        self.edit_mode = self.temp_subject.teacher == "None";
                                    }
                                    Some(2) => {
                                        if self.edit_buf.is_empty() {
                                            self.temp_subject.teacher = "None".to_string();
                                        } else {
                                            self.temp_subject.teacher = self.edit_buf.clone();
                                        }
                                        self.state_create_table.select_next();
                                        self.edit_mode = false;
                                    }
                                    _ => {
                                        self.edit_mode = false;
                                    }
                                }
                                self.edit_buf.clear();
                            }
                            KeyCode::Char(c) if c.is_ascii() => {
                                self.edit_buf.push(c);
                            }
                            KeyCode::Backspace => {
                                self.edit_buf.pop();
                            }
                            KeyCode::Esc => {
                                self.edit_buf.clear();
                                self.edit_mode = false;
                            }
                            _ => {}
                        },
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.create_popup = !self.create_popup;
                            self.edit_mode = false;
                            self.edit_buf.clear();
                        }
                        KeyCode::Tab => {
                            self.selected_create_tab = (self.selected_create_tab + 1) % 3
                        }
                        KeyCode::Up => {
                            self.state_create_table.select_previous();
                        }
                        KeyCode::Down => {
                            self.state_create_table.select_next();
                        }
                        KeyCode::Enter if self.state_create_table.selected() != Some(3) => {
                            match self.state_create_table.selected() {
                                Some(0) => {
                                    if self.temp_subject.name != "None" {
                                        self.edit_buf = self.temp_subject.name.clone();
                                    } else {
                                        self.edit_buf = String::new();
                                    }
                                }
                                Some(1) => {
                                    if self.temp_subject.room != "None" {
                                        self.edit_buf = self.temp_subject.room.clone();
                                    } else {
                                        self.edit_buf = String::new();
                                    }
                                }
                                Some(2) => {
                                    if self.temp_subject.teacher != "None" {
                                        self.edit_buf = self.temp_subject.teacher.clone();
                                    } else {
                                        self.edit_buf = String::new();
                                    }
                                }
                                _ => {}
                            }
                            self.edit_mode = true;
                        }
                        KeyCode::Enter if self.state_create_table.selected() == Some(3) => {
                            if let (Some(row_idx), Some(col_idx)) = (
                                self.state_table.selected(),
                                self.state_table.selected_column(),
                            ) {
                                self.set_selected_subject(
                                    row_idx,
                                    col_idx,
                                    self.temp_subject.clone(),
                                );
                            }
                            self.temp_subject = Subject::default();
                            self.create_popup = !self.create_popup;
                            self.state_create_table = TableState::default()
                                .with_selected(0)
                                .with_selected_column(1);
                            self.save_to_file()?;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    fn color_from_hex(n: String) -> Color {
        if n.len() != 7 {
            return Color::Black;
        }
        let mut hex = n.clone();
        let _x = hex.remove(0);
        let (r_hex, rest) = hex.split_at(2);
        let (g_hex, b_hex) = rest.split_at(2);
        let r = u8::from_str_radix(r_hex, 16).unwrap();
        let g = u8::from_str_radix(g_hex, 16).unwrap();
        let b = u8::from_str_radix(b_hex, 16).unwrap();
        Color::Rgb(r, g, b)
    }

    fn contrasting_fg(bg: Color) -> Color {
        if let Color::Rgb(r, g, b) = bg {
            let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            if luminance > 128.0 {
                Color::Black
            } else {
                Color::White
            }
        } else {
            Color::White
        }
    }

    fn save_to_file(&self) -> io::Result<()> {
        let save_data = SaveData {
            times: self.times.clone(),
            content: self.content.clone(),
        };
        let json = serde_json::to_string_pretty(&save_data)?;
        fs::write(&self.save_file, json)
    }

    fn get_selected_subject(&self, row_idx: usize, col_idx: usize) -> Option<&Subject> {
        let row_num = row_idx as u8 + 1;
        let column = COLUMNS[col_idx];

        self.content
            .iter()
            .find(|(r, c, _)| *r == row_num && *c == column)
            .map(|(_, _, subject)| subject)
    }

    fn set_selected_subject(&mut self, row_idx: usize, col_idx: usize, subject: Subject) {
        let row_num = row_idx as u8 + 1;
        let column = COLUMNS[col_idx];

        if let Some(entry) = self
            .content
            .iter_mut()
            .find(|(r, c, _)| *r == row_num && *c == column)
        {
            entry.2 = subject;
        }
    }

    fn render(&mut self, frame: &mut Frame, selected_tab: usize) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .spacing(Spacing::Overlap(1));
        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(6),
        ]);
        let settings = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [top, main, bottom] = frame.area().layout(&layout);
        let [left, _middle, right] = top.layout(&horizontal);
        let [_settings_top, settings_main, _settings_bottom] = main.layout(&settings);
        self.render_tabs(frame, left, selected_tab);
        self.render_info(frame, right);
        self.render_keybinds(frame, bottom);

        // let selected = self
        //     .state_settings
        //     .selected()
        //     .unwrap_or(0)
        //     .try_into()
        //     .unwrap_or(0);
        // frame.render_widget(
        //     Paragraph::new(self.times[selected].1.to_line_string()),
        //     middle,
        // );
        // let last_time: String = self
        //     .edit_buf
        //     .chars()
        //     .skip(self.edit_buf.chars().count().saturating_sub(5))
        //     .collect();
        // frame.render_widget(Paragraph::new(last_time), _middle);

        match self.selected_tab {
            0 => {
                self.render_table(frame, main);
                self.render_scroll_indicators(frame, main);
                if self.create_popup {
                    self.render_create_popup(frame, main);
                }
            }
            1 => self.render_settings(frame, settings_main),
            2 => {}
            _ => {}
        }
    }

    fn render_tabs(&mut self, frame: &mut Frame, area: Rect, selected_tab: usize) {
        let tabs = Tabs::new(vec!["Timetable", "Settings", "Details"])
            .highlight_style(Style::default().magenta().on_black().bold())
            .select(selected_tab)
            .divider(symbols::DOT)
            .padding(" ", " ")
            .block(
                Block::new()
                    .borders(Borders::BOTTOM | Borders::TOP | Borders::LEFT)
                    .merge_borders(MergeStrategy::Exact),
            );
        frame.render_widget(tabs, area);
    }

    fn render_info(&mut self, frame: &mut Frame, area: Rect) {
        let info = if self.edit_mode {
            "Edit".to_string()
        } else {
            "".to_string()
        };
        frame.render_widget(
            Paragraph::new(info).block(
                Block::new()
                    .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
                    .merge_borders(MergeStrategy::Exact),
            ),
            area,
        );
    }

    fn render_keybinds(&mut self, frame: &mut Frame, area: Rect) {
        let instructions = Line::from(vec![
            "Move ".into(),
            "<Arrow Keys>".blue().bold(),
            " Modify ".into(),
            "<Enter>".blue().bold(),
            " Clear ".into(),
            "<c>".blue().bold(),
            " Switch Tab ".into(),
            "<Tab>".blue().bold(),
            " Quit ".into(),
            "<Q> <Esc>".blue().bold(),
        ])
        .centered();
        frame.render_widget(
            Paragraph::new(instructions)
                .block(Block::bordered().merge_borders(MergeStrategy::Exact)),
            area,
        );
    }

    fn render_create_popup(&mut self, frame: &mut Frame, area: Rect) {
        let tab_titles = ["Create", "Choose", "Placeholder"];
        let popup_tabs = Tabs::new(tab_titles)
            .padding("", "")
            .highlight_style(Style::default().magenta().on_black())
            .select(self.selected_create_tab);
        let centered_area = area.centered(Constraint::Percentage(40), Constraint::Percentage(20));
        frame.render_widget(Clear, centered_area);
        let tabs_width: u16 = tab_titles.iter().map(|s| s.len() as u16).sum::<u16>() + 3 * 2;
        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(tabs_width),
            Constraint::Fill(1),
        ]);
        let [_left, tabs_area, _right] = centered_area.layout(&horizontal);

        let popup_block = Block::bordered();
        match self.selected_create_tab {
            0 => {
                self.render_popup_create_table(frame, centered_area, popup_block);
            }
            1 => {
                frame.render_widget(popup_block, centered_area);
            }
            2 => {
                frame.render_widget(popup_block, centered_area);
            }
            _ => {}
        }

        frame.render_widget(popup_tabs, tabs_area);
    }

    fn render_popup_create_table(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [_top, center, _bottom] = area.layout(&vertical);
        let horizontal = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(8),
        ]);
        let [_right, main_right, main_left, _left] = center.layout(&horizontal);
        let is_editing = self.edit_mode;
        let rows = [
            Row::new([
                "Name:".to_string(),
                if is_editing && self.state_create_table.selected() == Some(0) {
                    format!("{}_", self.edit_buf)
                } else {
                    self.temp_subject.name.clone()
                },
            ]),
            Row::new([
                "Room:".to_string(),
                if is_editing && self.state_create_table.selected() == Some(1) {
                    format!("{}_", self.edit_buf)
                } else {
                    self.temp_subject.room.clone()
                },
            ]),
            Row::new([
                "Teacher:".to_string(),
                if is_editing && self.state_create_table.selected() == Some(2) {
                    format!("{}_", self.edit_buf)
                } else {
                    self.temp_subject.teacher.clone()
                },
            ]),
            Row::new(["", "Submit"]),
        ];
        let table = Table::new(rows, [Constraint::Length(9), Constraint::Fill(1)])
            .cell_highlight_style(Style::new().magenta().not_dim());
        frame.render_stateful_widget(table, main_right, &mut self.state_create_table);
        frame.render_widget(block, area);
        let bg_color = App::color_from_hex(self.color.clone());
        frame.render_widget(
            Paragraph::new(self.color.clone())
                .fg(App::contrasting_fg(bg_color))
                .block(Block::new().bg(bg_color)),
            //Block::new().bg(App::color_from_hex(self.color.clone())),
            main_left,
        );
    }

    fn render_scroll_indicators(&mut self, frame: &mut Frame, area: Rect) {
        let row_height = 2u16;
        let border_lines = 2u16;
        let header_lines = 1u16;
        let visible_rows = (area.height.saturating_sub(border_lines + header_lines)) / row_height;

        let offset = self.state_table.offset();
        let has_above = offset > 0;
        let has_below = offset + visible_rows as usize + 1 < self.times.len();

        if has_above {
            frame.render_widget(
                Paragraph::new("▲"),
                Rect::new(area.x + area.width - 1, area.y + 1 + header_lines, 1, 1),
            );
        }
        if has_below {
            frame.render_widget(
                Paragraph::new("▼"),
                Rect::new(area.x + area.width - 1, area.y + area.height - 2, 1, 1),
            );
        }
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let selected_row = self.state_table.selected();
        let selected_col = self.state_table.selected_column();

        let rows: Vec<Row> = (1u8..=11)
            .map(|row_num| {
                let time_cell = Cell::from(self.times[row_num as usize - 1].1.to_line_string());

                let subject_cells: Vec<Cell> = COLUMNS
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| **c != Column::Time)
                    .map(|(col_idx, &column)| {
                        let (_, _, subject) = self
                            .content
                            .iter()
                            .find(|(r, c, _)| *r == row_num && *c == column)
                            .unwrap();

                        let is_editing = self.edit_mode
                            && selected_row == Some((row_num) as usize - 1)
                            && selected_col == Some(col_idx)
                            && !self.create_popup;

                        if is_editing {
                            Cell::from(format!("{}_", self.edit_buf))
                        } else {
                            Cell::from(subject.to_text())
                        }
                    })
                    .collect();

                let mut cells = vec![time_cell];
                cells.extend(subject_cells);
                let rows = Row::new(cells).height(2);
                if row_num % 2 == 0 {
                    rows.bg(Color::Rgb(30, 30, 30))
                } else {
                    rows.bg(Color::Rgb(10, 10, 10))
                }
            })
            .collect();

        let day_row = Row::new(["", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]);

        let mut constraints = vec![Constraint::Length(7)];
        constraints.extend([Constraint::Fill(1); 5]);

        let table = Table::new(rows, constraints)
            .cell_highlight_style(Style::new().magenta().not_dim())
            .block(Block::bordered().merge_borders(MergeStrategy::Exact))
            .row_highlight_style(Style::new().bg(Color::Rgb(0, 0, 50)))
            .header(day_row.bold().bg(Color::Rgb(30, 30, 30)));

        frame.render_stateful_widget(table, area, &mut self.state_table);
    }

    fn render_settings(&mut self, frame: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .times
            .iter()
            .enumerate()
            .map(|(i, (row, period))| {
                let is_editing = self.edit_mode && self.state_settings.selected() == Some(i);

                let period_cell = if is_editing {
                    Cell::from(format!("{}_", self.edit_buf))
                } else {
                    Cell::from(period.to_text())
                };

                Row::new([Cell::from(format!("{:02}", row)), period_cell])
            })
            .collect();

        let table = Table::new(rows, [Constraint::Length(3), Constraint::Fill(1)])
            .cell_highlight_style(Style::new().magenta())
            .block(Block::bordered().padding(Padding::horizontal(1)));

        let table_height = self.times.len() as u16 + 2;
        let table_width = 19;
        let vertical = Layout::vertical([Constraint::Length(table_height), Constraint::Fill(1)]);
        let [row_area, _rest] = area.layout(&vertical);
        let horizontal = Layout::horizontal([Constraint::Length(table_width), Constraint::Fill(1)]);
        let [table_area, _right] = row_area.layout(&horizontal);
        frame.render_stateful_widget(table, table_area, &mut self.state_settings);
    }
}
