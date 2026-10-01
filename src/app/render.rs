use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Rows, Spacing},
    style::{Color, Style, Stylize},
    symbols::{self, merge::MergeStrategy},
    text::Line,
    widgets::{Block, Borders, Cell, Clear, Padding, Paragraph, Row, Table, TableState, Tabs},
};

use crate::{
    app::App,
    structs::{COLUMNS, Column},
};

impl App {
    pub(crate) fn render(&mut self, frame: &mut Frame, selected_tab: usize) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .spacing(Spacing::Overlap(1));
        #[cfg(debug_assertions)]
        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(6),
        ]);
        #[cfg(not(debug_assertions))]
        let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Length(6)]);
        let settings = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [top, main, bottom] = frame.area().layout(&layout);
        #[cfg(debug_assertions)]
        let [left, _middle, right] = top.layout(&horizontal);
        #[cfg(not(debug_assertions))]
        let [left, right] = top.layout(&horizontal);
        let [_settings_top, settings_main, _settings_bottom] = main.layout(&settings);
        self.render_tabs(frame, left, selected_tab);
        self.render_info(frame, right);
        self.render_keybinds(frame, bottom);

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
        let horizontal_center = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [_cent_left, center_center, _cent_right] = center.layout(&horizontal_center);
        let horizontal = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(1),
        ]);
        let [_left, _main_left, col_right, _space_right, _right] = center.layout(&horizontal);
        let is_editing = self.edit_mode;
        let [red, green, blue] = App::color_to_parts(self.color);
        let rows = [
            Row::new([
                "Name:".to_string(),
                if is_editing && self.state_create_table.selected() == Some(0) {
                    format!("{}_ ", self.edit_buf)
                } else {
                    self.temp_subject.name.clone()
                },
                "#".to_string(),
                red,
                green,
                blue,
                "".to_string(),
                "".to_string(),
            ]),
            Row::new([
                "Room:".to_string(),
                if is_editing && self.state_create_table.selected() == Some(1) {
                    format!("{}_ ", self.edit_buf)
                } else {
                    self.temp_subject.room.clone()
                },
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]),
            Row::new([
                "Teacher:".to_string(),
                if is_editing && self.state_create_table.selected() == Some(2) {
                    format!("{}_ ", self.edit_buf)
                } else {
                    self.temp_subject.teacher.clone()
                },
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]),
            Row::new(["", "Submit", "", "", "", "", "", ""]),
        ];
        let table = Table::new(
            rows,
            [
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(8),
            ],
        )
        .cell_highlight_style(Style::new().magenta().not_dim())
        .column_spacing(0);
        frame.render_widget(block, area);
        let bg_color = self.color;
        frame.render_widget(
            Paragraph::default()
                .fg(App::contrasting_fg(bg_color))
                .block(Block::new().bg(bg_color)),
            col_right,
        );
        frame.render_stateful_widget(table, center_center, &mut self.state_create_table);
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
        let horizontal =
            Layout::horizontal([Constraint::Length(19), Constraint::Fill(1)]).spacing(1);
        let [right, left] = area.layout(&horizontal);
        self.render_settings_time(frame, right);
        self.render_settings_autotime(frame, left);
    }

    fn render_settings_time(&mut self, frame: &mut Frame, area: Rect) {
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
        let vertical = Layout::vertical([Constraint::Length(table_height), Constraint::Fill(1)]);
        let [row_area, _rest] = area.layout(&vertical);
        frame.render_stateful_widget(table, row_area, &mut self.state_settings);
    }

    fn render_settings_autotime(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Table::new([Row::new(["dad"])], [Constraint::Fill(1)]).block(Block::bordered()),
            area,
            &mut TableState::new(),
        );
    }
}
