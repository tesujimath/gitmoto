use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, List, Paragraph},
    Frame,
};

use crate::model::Model;

/// Renders the user interface widgets.
pub fn render(model: &mut Model, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    frame.render_widget(
        List::new(model.repos.iter().map(|repo| repo.path.to_string_lossy())),
        // Paragraph::new(
        //     "This is a work-in-progress.\n\
        //         Press `Esc`, `Ctrl-C` or `q` to stop running."
        //         .to_string(),
        // )
        // .block(
        //     Block::bordered()
        //         .title("Template")
        //         .title_alignment(Alignment::Center)
        //         .border_type(BorderType::Rounded),
        // )
        // .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        // .centered(),
        frame.area(),
    )
}
