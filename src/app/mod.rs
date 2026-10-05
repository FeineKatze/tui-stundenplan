use std::{fs, io, path::Path};

use ratatui::{DefaultTerminal, style::Color, widgets::TableState};

use crate::{
    period::Period,
    structs::{COLUMNS, Column, SaveData},
    subject::Subject,
};

mod functions;
mod input;
mod render;

#[allow(dead_code)]
pub struct App {
    pub selected_tab: usize,
    pub save_file: String,
    pub times: Vec<(u8, Period)>,
    pub state_table: TableState,
    pub state_settings: TableState,
    pub edit_mode: bool,
    pub edit_buf: String,
    pub content: Vec<(u8, Column, Subject)>,
    pub create_popup: bool,
    pub selected_create_tab: usize,
    pub state_create_table: TableState,
    pub temp_subject: Subject,
    pub index_subjects: Vec<String>,
    pub index_rooms: Vec<String>,
    pub index_teachers: Vec<String>,
    pub color: Color,
    pub blend_highlight: bool,
    pub state_toggles: TableState,
}

impl App {
    pub fn new() -> Self {
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
            color: Color::Rgb(0, 0, 0),
            blend_highlight: false,
            state_toggles: TableState::default()
                .with_selected(None)
                .with_selected_column(Some(0)),
        }
    }

    pub(crate) fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
            if self.handle_key()? {
                return Ok(());
            }
        }
    }
}
