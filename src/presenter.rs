use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use dirs::home_dir;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin},
    style::Modifier,
    widgets::{
        Block, BorderType, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    },
    Frame,
};
use std::{
    borrow::Cow,
    cmp::{max, min},
    default::Default,
    path::{Path, PathBuf},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::model::{LocalRepo, Model, UpdateModel};

#[derive(Debug)]
pub struct Presenter {
    home_dir: Option<String>,
    model: Model,
    repo_filter_input: Input,
    view_height: usize,
    selected: Option<Selected>,
}

impl Default for Presenter {
    fn default() -> Self {
        let model = Model::default();
        Self {
            home_dir: home_dir().map(|p| p.to_string_lossy().into_owned()),
            model,
            repo_filter_input: Input::default(),
            view_height: 1,
            selected: None,
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
        } else if ev.code == KeyCode::Up {
            self.scroll(-1);
        } else if ev.code == KeyCode::Down {
            self.scroll(1);
        } else if ev.code == KeyCode::PageUp {
            self.scroll(-(self.view_height as isize));
        } else if ev.code == KeyCode::PageDown {
            self.scroll(self.view_height as isize);
        } else if ev.code == KeyCode::Enter {
            self.open_git_client();
        } else {
            self.repo_filter_input.handle_event(&Event::Key(ev));
        }

        false
    }

    fn scroll(&mut self, offset: isize) {
        let (filtered_repos, u_selected) = self.filtered_repos();
        if !filtered_repos.is_empty() {
            let filtered_repos_len = filtered_repos.len();

            let u_scrolled = match u_selected {
                Some(u_selected) => {
                    constrained_by((u_selected as isize) + offset, filtered_repos_len)
                }
                None => constrained_by(offset, filtered_repos_len),
            };

            let u_view_scrolled = match (self.selected.as_ref(), u_selected) {
                (Some(selected), Some(u_selected)) => {
                    let i_view = selected.u_view as isize;
                    if u_scrolled > u_selected
                        && selected.u_view + u_scrolled - u_selected < self.view_height
                    {
                        selected.u_view + (u_scrolled - u_selected)
                    } else if i_view + offset >= 0 && i_view + offset < self.view_height as isize {
                        (i_view + offset) as usize
                    } else {
                        selected.u_view
                    }
                }

                _ => {
                    if offset < 0 {
                        0
                    } else {
                        let offset = offset as usize;
                        if offset < self.view_height {
                            offset
                        } else {
                            0
                        }
                    }
                }
            };

            self.selected = Some(Selected::new(
                filtered_repos[u_scrolled].path.clone(),
                min(u_view_scrolled, u_scrolled),
            ));
        }
    }

    fn filtered_repos(&self) -> (Vec<&LocalRepo>, Option<usize>) {
        let filters = self
            .repo_filter_input
            .value()
            .split(' ')
            .collect::<Vec<_>>();
        let mut u_selected = None;
        let repos = self
            .model
            .repos
            .iter()
            .map(|(path, repo)| (path.to_string_lossy(), repo))
            .filter(move |(s, _)| filters.iter().all(|f| s.contains(f)))
            .enumerate()
            .map(|(i, (_, repo))| {
                if self
                    .selected
                    .as_ref()
                    .is_some_and(|selected| selected.path == repo.path)
                {
                    u_selected = Some(i);
                }

                repo
            })
            .collect();

        (repos, u_selected)
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let main_layout =
            Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).split(frame.area());
        let repo_layout = Layout::horizontal(vec![Constraint::Length(1), Constraint::Fill(1)])
            .split(main_layout[1]);

        const BORDER_WASTAGE: usize = 2;
        self.view_height = (main_layout[1].height as usize).saturating_sub(BORDER_WASTAGE);

        frame.render_widget(
            Paragraph::new(self.repo_filter_input.to_string()),
            main_layout[0],
        );

        let (filtered_repos, u_selected) = self.filtered_repos();
        let n_filtered_repos = filtered_repos.len();

        // work out what is visible
        let max_visible = main_layout[1].height as usize;
        let skip = if let Some(u_selected) = u_selected {
            u_selected - min(self.selected.as_ref().unwrap().u_view, u_selected)
        } else {
            0
        };

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let scrollbar_content_length = if n_filtered_repos < self.view_height {
            n_filtered_repos
        } else {
            n_filtered_repos - self.view_height
        };
        let mut scrollbar_state = ScrollbarState::new(scrollbar_content_length).position(skip);

        let table_widths = [
            Constraint::Ratio(1, 2),
            Constraint::Length(10),
            Constraint::Ratio(1, 2),
        ];

        let rows = filtered_repos
            .into_iter()
            .enumerate()
            .skip(skip)
            .take(max_visible)
            .map(|(i, repo)| {
                let modifier = if u_selected.is_some_and(|selected| i == selected) {
                    Modifier::REVERSED
                } else {
                    Modifier::default()
                };
                Row::new([
                    self.display_path(&repo.path),
                    Cow::Owned(repo.remotes.len().to_string()),
                    Cow::Borrowed(""),
                ])
                .style(modifier)
            })
            .collect::<Vec<_>>();

        frame.render_stateful_widget(
            scrollbar,
            repo_layout[0].inner(Margin {
                // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );

        frame.render_widget(
            Table::new(rows, table_widths).block(
                Block::bordered()
                    .title(format!(
                        " filtered {}/{} local repos ",
                        n_filtered_repos,
                        self.model.repos.len()
                    ))
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            ),
            repo_layout[1],
        )
    }

    fn display_path<'a>(&self, path: &'a Path) -> Cow<'a, str> {
        let path = path.to_string_lossy();
        if let Some(home_dir) = self.home_dir.as_ref() {
            if path.starts_with(home_dir) {
                return Cow::Owned(path.replacen(home_dir, "~", 1));
            }
        }
        path
    }

    // TODO - this shouldn't be inline perhaps?
    fn open_git_client(&mut self) {
        if let Some(selected) = self.selected.as_ref() {
            let path = selected.path.canonicalize().unwrap();
            let magit_status = format!("(magit-status \"{}\")", path.to_string_lossy());
            std::process::Command::new("emacsclient")
                .args(["--create-frame", "--eval", &magit_status])
                .spawn()
                .expect("Failed to spawn emacsclient");
        }
    }

    // fn model_updated(&mut self) {
    //     trace!("model updated");
    // }
}

impl UpdateModel for Presenter {
    fn add_local_repo(&mut self, repo: LocalRepo) {
        self.model.add_local_repo(repo);
        // self.model_updated();
    }
}

#[derive(Debug)]
struct Selected {
    path: PathBuf,
    u_view: usize,
}

impl Selected {
    fn new(path: PathBuf, u_view: usize) -> Self {
        Self { path, u_view }
    }
}

fn is_quit(key_event: &KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL)
}

/// constrain i in [0, limit), panic if limit is zero
fn constrained_by(i: isize, limit: usize) -> usize {
    assert!(limit > 0);
    min(max(0, i) as usize, limit - 1)
}
