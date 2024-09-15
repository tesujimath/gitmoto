use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, BorderType, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    const DEFAULT_PATH_LEN: u16 = 20u16;
    let max_len = app
        .repos
        .keys()
        .map(|path| path.as_os_str().len() as u16)
        .max()
        .unwrap_or(DEFAULT_PATH_LEN);
    // let layout =
    //     Layout::horizontal([Constraint::Length(max_len), Constraint::Fill(1)]).split(frame.area());
    let widths = [Constraint::Length(max_len)];

    let layout =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).split(frame.area());

    frame.render_widget(Paragraph::new(app.repo_filter_input.to_string()), layout[0]);

    let rows = app
        .filtered_repos()
        .map(|repo| Row::new([repo.path.to_string_lossy()]))
        .collect::<Vec<_>>();
    let n_rows = rows.len();

    frame.render_widget(
        Table::new(rows, widths).block(
            Block::bordered()
                .title(format!(
                    " showing {}/{} local repos ",
                    n_rows,
                    app.repos.len()
                ))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        ),
        layout[1],
    )
}
