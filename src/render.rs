use std::{
    borrow::Cow,
    iter::{once, repeat},
};

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, BorderType, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    const DEFAULT_NAME_LEN: u16 = 4u16;
    // let repo_max_len = app
    //     .repos
    //     .keys()
    //     .map(|path| path.as_os_str().len() as u16)
    //     .max()
    //     .unwrap_or(DEFAULT_PATH_LEN);
    let remote_name_max_len = app
        .repos
        .values()
        .flat_map(|repo| repo.remotes.iter().map(|remote| remote.name().len() as u16))
        .max()
        .unwrap_or(DEFAULT_NAME_LEN);
    // let layout =
    //     Layout::horizontal([Constraint::Length(max_len), Constraint::Fill(1)]).split(frame.area());
    // let widths = [Constraint::Length(repo_max_len), Constraint::Length(3)];

    let widths = [
        Constraint::Ratio(1, 2),
        Constraint::Length(10),
        Constraint::Ratio(1, 2),
    ];

    let layout =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).split(frame.area());

    frame.render_widget(Paragraph::new(app.repo_filter_input.to_string()), layout[0]);

    let filtered_repos = app.filtered_repos().collect::<Vec<_>>();
    let n_filtered_repos = filtered_repos.len();

    let rows = filtered_repos
        .into_iter()
        .flat_map(|repo| {
            let repo_path_then_empty =
                once(repo.path.to_string_lossy()).chain(repeat(Cow::Borrowed("")));
            repo_path_then_empty
                .zip(repo.remotes.iter())
                .map(|(path, remote)| {
                    Row::new([
                        path,
                        Cow::Borrowed(remote.name()),
                        Cow::Borrowed(remote.url()),
                    ])
                })
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Table::new(rows, widths).block(
            Block::bordered()
                .title(format!(
                    " showing {}/{} local repos ",
                    n_filtered_repos,
                    app.repos.len()
                ))
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        ),
        layout[1],
    )
}
