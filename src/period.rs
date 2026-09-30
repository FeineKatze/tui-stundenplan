use core::fmt;
use std::str::FromStr;

use ratatui::{
    style::Stylize,
    text::{Line, Span, Text},
};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Period {
    pub first_time: String,
    pub second_time: String,
}

impl Period {
    pub fn default() -> Self {
        Self {
            first_time: "00:00".to_string(),
            second_time: "00:00".to_string(),
        }
    }

    pub fn to_line_string(&self) -> String {
        format!("{}-\n{}", self.first_time, self.second_time)
    }

    pub fn to_text(&self) -> Text<'static> {
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
