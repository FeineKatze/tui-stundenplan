use std::io;

use crossterm::event::{self, KeyCode};
use ratatui::{style::Color, widgets::TableState};

use crate::{app::App, period::Period, subject::Subject};

impl App {
    pub(crate) fn handle_key(&mut self) -> io::Result<bool> {
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                _ if !self.create_popup => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc if !self.edit_mode => return Ok(true),
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
                                self.state_create_table.select_column(Some(1));
                                self.color = Color::Rgb(0, 0, 0);
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
                                    self.set_selected_subject(row_idx, col_idx, Subject::default());
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
                                    if matches!(self.edit_buf.chars().last(), Some(':') | Some('-'))
                                    {
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
                    KeyCode::Tab => self.selected_create_tab = (self.selected_create_tab + 1) % 3,
                    KeyCode::Up => match self.state_create_table.selected_column() {
                        Some(1) => {
                            self.state_create_table.select_previous();
                        }
                        Some(3) => {
                            if let Color::Rgb(r, g, b) = self.color {
                                self.color = Color::Rgb((r as i16 + 1).clamp(0, 255) as u8, g, b);
                            }
                        }
                        Some(4) => {
                            if let Color::Rgb(r, g, b) = self.color {
                                self.color = Color::Rgb(r, (g as i16 + 1).clamp(0, 255) as u8, b);
                            }
                        }
                        Some(5) => {
                            if let Color::Rgb(r, g, b) = self.color {
                                self.color = Color::Rgb(r, g, (b as i16 + 1).clamp(0, 255) as u8);
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Down => match self.state_create_table.selected_column() {
                        Some(1) => {
                            self.state_create_table.select_next();
                        }
                        Some(3) => {
                            if let Color::Rgb(r, g, b) = self.color {
                                self.color = Color::Rgb((r as i16 - 1).clamp(0, 255) as u8, g, b);
                            }
                        }
                        Some(4) => {
                            if let Color::Rgb(r, g, b) = self.color {
                                self.color = Color::Rgb(r, (g as i16 - 1).clamp(0, 255) as u8, b);
                            }
                        }
                        Some(5) => {
                            if let Color::Rgb(r, g, b) = self.color {
                                self.color = Color::Rgb(r, g, (b as i16 - 1).clamp(0, 255) as u8);
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Right => {
                        if self.state_create_table.selected() == Some(0) {
                            match self.state_create_table.selected_column() {
                                Some(1) => {
                                    self.state_create_table.select_column(Some(3));
                                }
                                Some(3) => {
                                    self.state_create_table.select_column(Some(4));
                                }
                                Some(4) => {
                                    self.state_create_table.select_column(Some(5));
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Left => match self.state_create_table.selected_column() {
                        Some(5) => {
                            self.state_create_table.select_column(Some(4));
                        }
                        Some(4) => {
                            self.state_create_table.select_column(Some(3));
                        }
                        Some(3) => {
                            self.state_create_table.select_column(Some(1));
                        }
                        _ => {}
                    },
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
                            self.set_selected_subject(row_idx, col_idx, self.temp_subject.clone());
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
        Ok(false)
    }
}
