use std::{borrow::Cow, iter::once};

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, BorderType, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    // const DEFAULT_PATH_LEN: u16 = 20u16;
    // let repo_max_len = app
    //     .repos
    //     .keys()
    //     .map(|path| path.as_os_str().len() as u16)
    //     .max()
    //     .unwrap_or(DEFAULT_PATH_LEN);
    // let remote_max_len = app
    //     .repos
    //     .values()
    //     .flat_map(|repo| repo.remotes)
    //     .unwrap_or(DEFAULT_PATH_LEN);
    // let layout =
    //     Layout::horizontal([Constraint::Length(max_len), Constraint::Fill(1)]).split(frame.area());
    // let widths = [Constraint::Length(repo_max_len), Constraint::Length(3)];
    let widths = [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)];

    let layout =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).split(frame.area());

    frame.render_widget(Paragraph::new(app.repo_filter_input.to_string()), layout[0]);

    let rows = app
        .filtered_repos()
        .flat_map(|repo| {
            let mut remote_urls = repo.remotes.iter().map(|remote| remote.url());
            once(Row::new([
                repo.path.to_string_lossy(),
                Cow::Borrowed(remote_urls.by_ref().next().unwrap_or("")),
            ]))
            .chain(remote_urls.map(|url| Row::new(["", url])))
        })
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
