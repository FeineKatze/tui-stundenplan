use std::str::FromStr;

use ratatui::{
    style::{Color, Stylize},
    text::{Line, Text},
};
use serde::{Deserialize, Serialize};

fn default_color() -> Color {
    Color::Reset
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub room: String,
    pub teacher: String,
    #[serde(default = "default_color")]
    pub color: Color,
}

impl Subject {
    pub fn default() -> Self {
        Self {
            name: String::from("None"),
            room: String::from("None"),
            teacher: String::from("None"),
            color: Color::Reset,
        }
    }

    pub fn to_text(&self) -> Text<'static> {
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
            color: Color::Reset,
        })
    }
}
