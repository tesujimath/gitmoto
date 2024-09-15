use std::{collections::BTreeMap, default::Default, path::PathBuf};

use crossterm::event::{Event, KeyEvent};
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
        self.repo_filter_input.handle_event(&Event::Key(ev));
    }

    pub fn add_local_repo(&mut self, path: PathBuf) {
        self.repos.insert(path.clone(), LocalRepo::new(path));
    }
}

#[derive(Debug)]
pub struct LocalRepo {
    pub path: PathBuf,
    pub remotes: Option<Vec<Remote>>, // None means unknown, empty vec means no remotes
}

impl LocalRepo {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            remotes: None,
        }
    }
}

#[derive(Debug)]
pub struct Remote {
    url: String,
}
