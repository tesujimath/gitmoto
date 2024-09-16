use crossterm::event::{Event, KeyCode, KeyEvent};
use std::default::Default;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::model::{LocalRepo, Model};

#[derive(Debug)]
pub struct Presenter {
    pub model: Model,
    pub repo_filter_input: Input,
}

impl Presenter {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            repo_filter_input: Input::default(),
        }
    }
    pub fn key(&mut self, ev: KeyEvent) {
        if ev.code == KeyCode::Esc {
            self.repo_filter_input.reset();
        } else {
            self.repo_filter_input.handle_event(&Event::Key(ev));
        }
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
}
