use std::{default::Default, path::PathBuf};

#[derive(Debug)]
pub struct Model {
    pub running: bool,
    pub repo_filter: String,
    pub repos: Vec<LocalRepo>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running: true,
            repo_filter: "rust".to_string(), //String::default(),
            repos: Vec::default(),
        }
    }
}

impl Model {
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn add_local_repo(&mut self, path: PathBuf) {
        self.repos.push(LocalRepo::new(path))
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
