use crate::app::state::App;
use crate::ui::helpers::centered_rect;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_update_popup(f: &mut Frame, app: &App, area: Rect) {
    if !app.show_update_popup {
        return;
    }

    let version = app.update_available_version.as_deref().unwrap_or("Unknown");
    let popup_area = centered_rect(40, 35, area);

    let block = Block::default()
        .title("Update Available")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let inner_area = block.inner(popup_area);

    f.render_widget(Clear, popup_area); // Clear background
    f.render_widget(block, popup_area);

    let text = format!(
        "\n\nA new version ({}) is available!\n\nVisit GitHub releases to download.\n\nPress <Enter> or 'o' to open link.\nPress any other key to close.",
        version
    );

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, inner_area);
}
