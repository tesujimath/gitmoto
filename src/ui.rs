use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, BorderType, Paragraph, Row, Table},
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
    // let layout =
    //     Layout::horizontal([Constraint::Length(max_len), Constraint::Fill(1)]).split(frame.area());
    let widths = [Constraint::Length(max_len)];

    let layout =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).split(frame.area());

    frame.render_widget(Paragraph::new(model.repo_filter.clone()), layout[0]);

    let rows = model
        .repos
        .iter()
        .map(|repo| (repo.path.to_string_lossy(), repo))
        .filter(|(s, _)| s.contains(&model.repo_filter))
        .map(|(s, _)| Row::new([s]))
        .collect::<Vec<_>>();
    let n_rows = rows.len();

    frame.render_widget(
        Table::new(rows, widths).block(
            Block::bordered()
                .title(format!(
                    "showing {}/{} local repos",
                    n_rows,
                    model.repos.len()
                ))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        ),
        layout[1],
    )
}
