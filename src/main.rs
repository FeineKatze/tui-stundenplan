use std::io::{self};

use crate::app::App;

mod app;
mod period;
mod structs;
mod subject;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}
