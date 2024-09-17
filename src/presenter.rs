use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, BorderType, Paragraph, Row, Table},
    Frame,
};
use std::{
    borrow::Cow,
    default::Default,
    iter::{once, repeat},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::model::{LocalRepo, Model, UpdateModel};

#[derive(Debug)]
pub struct Presenter {
    model: Model,
    repo_filter_input: Input,
}

impl Default for Presenter {
    fn default() -> Self {
        let model = Model::default();
        Self {
            model,
            repo_filter_input: Input::default(),
        }
    }
}

impl Presenter {
    pub fn handle_key(&mut self, ev: KeyEvent) -> bool {
        if is_quit(&ev) {
            return true;
        }

        if ev.code == KeyCode::Esc {
            self.repo_filter_input.reset();
        } else {
            self.repo_filter_input.handle_event(&Event::Key(ev));
        }

        false
    }

    pub fn filtered_repos(&self) -> impl Iterator<Item = &LocalRepo> {
        let filters = self
            .repo_filter_input
            .value()
            .split(' ')
            .collect::<Vec<_>>();
        self.model
            .repos
            .iter()
            .map(|(path, repo)| (path.to_string_lossy(), repo))
            .filter(move |(s, _)| filters.iter().all(|f| s.contains(f)))
            .map(|(_, repo)| repo)
    }

    pub fn render(&self, frame: &mut Frame) {
        // const DEFAULT_PATH_LEN: u16 = 20u16;
        // let repo_max_len = app
        //     .repos
        //     .keys()
        //     .map(|path| path.as_os_str().len() as u16)
        //     .max()
        //     .unwrap_or(DEFAULT_PATH_LEN);

        // const DEFAULT_NAME_LEN: u16 = 4u16;
        // let remote_name_max_len = app
        //     .repos
        //     .values()
        //     .flat_map(|repo| repo.remotes.iter().map(|remote| remote.name().len() as u16))
        //     .max()
        //     .unwrap_or(DEFAULT_NAME_LEN);

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

        frame.render_widget(
            Paragraph::new(self.repo_filter_input.to_string()),
            layout[0],
        );

        let filtered_repos = self.filtered_repos().collect::<Vec<_>>();
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
                        self.model.repos.len()
                    ))
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            ),
            layout[1],
        )
    }
}

impl UpdateModel for Presenter {
    fn add_local_repo(&mut self, repo: LocalRepo) {
        self.model.add_local_repo(repo);
    }
}

fn is_quit(key_event: &KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL)
}
