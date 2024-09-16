use std::{collections::BTreeMap, default::Default, fmt::Display, path::PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub repo_filter_input: Input,
    pub repos: BTreeMap<PathBuf, LocalRepo>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            repo_filter_input: Input::default(),
            repos: BTreeMap::default(),
        }
    }
}

impl App {
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn key(&mut self, ev: KeyEvent) {
        if ev.code == KeyCode::Esc {
            self.repo_filter_input.reset();
        } else {
            self.repo_filter_input.handle_event(&Event::Key(ev));
        }
    }

    pub fn add_local_repo(&mut self, repo: LocalRepo) {
        self.repos.insert(repo.path.clone(), repo);
    }

    pub fn filtered_repos(&self) -> impl Iterator<Item = &LocalRepo> {
        let filters = self
            .repo_filter_input
            .value()
            .split(' ')
            .collect::<Vec<_>>();
        self.repos
            .iter()
            .map(|(path, repo)| (path.to_string_lossy(), repo))
            .filter(move |(s, _)| filters.iter().all(|f| s.contains(f)))
            .map(|(_, repo)| repo)
    }
}

#[derive(Debug)]
pub struct LocalRepo {
    pub path: PathBuf,
    pub remotes: Vec<Remote>,
}

impl LocalRepo {
    pub fn new(path: PathBuf, remotes: Vec<Remote>) -> Self {
        Self { path, remotes }
    }
}

#[derive(Debug)]
pub struct Remote {
    url: String,
}

impl Remote {
    pub fn new<S>(url: S) -> Self
    where
        S: Display,
    {
        Self {
            url: url.to_string(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
