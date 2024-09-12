use std::{default::Default, path::PathBuf};

#[derive(Debug)]
pub struct Model {
    pub running: bool,
    pub repos: Vec<LocalRepo>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running: true,
            repos: vec![
                LocalRepo::new(PathBuf::from("dummy/path/1")),
                LocalRepo::new(PathBuf::from("dummy/path/2")),
            ],
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
