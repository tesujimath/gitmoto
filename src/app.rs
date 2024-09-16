use crossterm::event::KeyEvent;
use std::default::Default;

use crate::{model::Model, presenter::Presenter};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub presenter: Presenter,
}

impl Default for App {
    fn default() -> Self {
        let model = Model::default();
        Self {
            running: true,
            presenter: Presenter::new(model),
        }
    }
}

impl App {
    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn key(&mut self, ev: KeyEvent) {
        self.presenter.key(ev)
    }
}
