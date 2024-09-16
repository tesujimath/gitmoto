use std::{collections::BTreeMap, default::Default, fmt::Display, path::PathBuf};

#[derive(Default, Debug)]
pub struct Model {
    pub repos: BTreeMap<PathBuf, LocalRepo>,
}

impl Model {
    pub fn add_local_repo(&mut self, repo: LocalRepo) {
        self.repos.insert(repo.path.clone(), repo);
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
    name: String,
    url: String,
}

impl Remote {
    pub fn new<S1, S2>(name: S1, url: S2) -> Self
    where
        S1: Display,
        S2: Display,
    {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
