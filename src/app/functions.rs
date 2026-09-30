use std::{fs, io};

use ratatui::style::Color;

use crate::{
    app::App,
    structs::{COLUMNS, SaveData},
    subject::Subject,
};

impl App {
    pub(crate) fn color_to_parts(col: Color) -> [String; 3] {
        if let Color::Rgb(r, g, b) = col {
            [
                format!("{:02x}", r),
                format!("{:02x}", g),
                format!("{:02x}", b),
            ]
        } else {
            ["00".to_string(), "00".to_string(), "00".to_string()]
        }
    }

    pub(crate) fn contrasting_fg(bg: Color) -> Color {
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

    pub(crate) fn save_to_file(&self) -> io::Result<()> {
        let save_data = SaveData {
            times: self.times.clone(),
            content: self.content.clone(),
        };
        let json = serde_json::to_string_pretty(&save_data)?;
        fs::write(&self.save_file, json)
    }

    pub(crate) fn get_selected_subject(&self, row_idx: usize, col_idx: usize) -> Option<&Subject> {
        let row_num = row_idx as u8 + 1;
        let column = COLUMNS[col_idx];

        self.content
            .iter()
            .find(|(r, c, _)| *r == row_num && *c == column)
            .map(|(_, _, subject)| subject)
    }

    pub(crate) fn set_selected_subject(
        &mut self,
        row_idx: usize,
        col_idx: usize,
        subject: Subject,
    ) {
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
}
