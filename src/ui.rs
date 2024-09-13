use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, BorderType, List},
    Frame,
};

use crate::model::Model;

pub fn render(model: &mut Model, frame: &mut Frame) {
    const DEFAULT_PATH_LEN: u16 = 20u16;
    let max_len = model
        .repos
        .iter()
        .map(|repo| repo.path.as_os_str().len() as u16)
        .max()
        .unwrap_or(DEFAULT_PATH_LEN);
    let layout =
        Layout::horizontal([Constraint::Length(max_len), Constraint::Fill(1)]).split(frame.area());

    frame.render_widget(
        List::new(model.repos.iter().map(|repo| repo.path.to_string_lossy())).block(
            Block::bordered()
                .title(format!("{} local repos", model.repos.len()))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        ),
        layout[0],
    )
}
