use crate::app::state::{App, AppMode};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let help_spans = match app.mode {
        AppMode::Normal => vec![
            Span::raw("↑↓ Nav | "),
            Span::styled(
                "a",
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Add | "),
            Span::styled(
                "e",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Edit | "),
            Span::styled(
                "d",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Del | "),
            Span::styled(
                "f",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Filt | "),
            Span::raw("s Mth | c Cate | "),
            Span::styled(
                "1-6",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Sort | "),
            Span::styled(
                "q/Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Quit | "),
            Span::styled("o", Style::default().fg(Color::Red)).add_modifier(Modifier::BOLD),
            Span::raw(" ⚙"),
        ],
        AppMode::Adding | AppMode::Editing => vec![
            Span::raw("Tab/↑↓ Nav | "),
            Span::raw("←→ Toggle | "),
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(" Save/Select | "),
            Span::styled("Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Cancel"),
        ],
        AppMode::ConfirmDelete => vec![
            Span::styled("y", Style::default().fg(Color::LightGreen)),
            Span::raw(": Confirm | "),
            Span::styled("n/Esc", Style::default().fg(Color::LightRed)),
            Span::raw(": Cancel"),
        ],
        AppMode::Filtering => vec![
            Span::raw("← → Cursor | "),
            Span::raw("Bksp/Del Edit | "),
            Span::styled("Ctrl+F", Style::default().fg(Color::Red)),
            Span::raw(" Adv Filt | "),
            Span::styled("Ctrl+R", Style::default().fg(Color::LightYellow)),
            Span::raw(" Clear | "),
            Span::styled("Enter/Esc", Style::default().fg(Color::LightGreen)),
            Span::raw(" Apply/Exit"),
        ],
        AppMode::AdvancedFiltering => vec![
            Span::raw("Tab/↑↓ Nav | "),
            Span::raw("← → Adjust | "),
            Span::styled("Ctrl+R", Style::default().fg(Color::LightYellow)),
            Span::raw(" Clear | "),
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(" Save | "),
            Span::styled("Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Cancel"),
        ],
        AppMode::SelectingFilterCategory | AppMode::SelectingFilterSubcategory => vec![
            Span::raw("↑↓ Nav | "),
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(": Confirm | "),
            Span::styled("Esc", Style::default().fg(Color::LightRed)),
            Span::raw(": Cancel"),
        ],
        AppMode::SelectingCategory | AppMode::SelectingSubcategory => vec![
            Span::raw("↑↓ Nav | "),
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(": Confirm | "),
            Span::styled("Esc", Style::default().fg(Color::LightRed)),
            Span::raw(": Cancel"),
        ],
        AppMode::Summary => vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Month | "),
            Span::styled(
                "←→",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/"),
            Span::styled(
                "[]",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Year | "),
            Span::styled("m", Style::default().fg(Color::LightBlue)),
            Span::raw(" Multi | "),
            Span::styled("q/Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Back"),
        ],
        AppMode::CategorySummary => vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Nav | "),
            Span::styled(
                "←→",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/"),
            Span::styled(
                "[]",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Year | "),
            Span::styled("Enter", Style::default().fg(Color::Magenta)),
            Span::raw(" ▶ Expand ▼ Collapse | "),
            Span::styled("q/Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Back"),
        ],
        AppMode::Settings => vec![
            Span::styled(
                "Tab/↑↓",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Nav | "),
            Span::styled(
                "←/→",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Cursor | "),
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(": Save | "),
            Span::styled("Esc", Style::default().fg(Color::LightRed)),
            Span::raw(": Cancel | "),
            Span::styled("Ctrl+D", Style::default().fg(Color::LightMagenta)),
            Span::raw(": Reset | "),
            Span::styled("Ctrl+U", Style::default().fg(Color::LightMagenta)),
            Span::raw(": Clear"),
        ],
    };

    let help_paragraph = Paragraph::new(Line::from(help_spans))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_paragraph, area);
}
