use serde::{Deserialize, Serialize};

use crate::{period::Period, subject::Subject};

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub times: Vec<(u8, Period)>,
    pub content: Vec<(u8, Column, Subject)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Column {
    Time,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

pub const COLUMNS: [Column; 6] = [
    Column::Time,
    Column::Monday,
    Column::Tuesday,
    Column::Wednesday,
    Column::Thursday,
    Column::Friday,
];
