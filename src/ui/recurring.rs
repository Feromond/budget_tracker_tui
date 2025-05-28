use crate::app::state::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_recurring_settings(f: &mut Frame, app: &App, area: Rect) {
    // Field definitions with titles and hints
    let field_definitions = [
        ("Is Recurring", "(◀/▶ to toggle)"),
        ("Frequency", "(Enter to select)"),
        (
            "End Date (YYYY-MM-DD)",
            "(Optional - ◀/▶ days, Shift+◀/▶ months, jumps to today if empty)",
        ),
    ];

    let input_widgets: Vec<_> = app
        .recurring_settings_fields
        .iter()
        .zip(field_definitions.iter())
        .enumerate()
        .map(|(i, (text, (base_title, hint)))| {
            let is_focused = app.current_recurring_field == i;
            let title = format!("{} {}", base_title, hint).trim_end().to_string();

            let content = match i {
                0 => {
                    // Is Recurring field - show as toggle
                    Span::styled(
                        format!(" < {} > ", text),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                }
                1 => {
                    // Frequency field - show as selection
                    Span::styled(
                        format!("  {}  ", text),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                }
                2 => {
                    // End Date field - show as text input
                    if text.is_empty() {
                        Span::styled(
                            " (Optional - leave empty for no end date) ",
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::ITALIC),
                        )
                    } else {
                        Span::raw(text.as_str())
                    }
                }
                _ => Span::raw(text.as_str()),
            };

            Paragraph::new(content)
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .border_style(if is_focused {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        }),
                )
        })
        .collect();

    // Vertical scroll logic for small terminal heights
    let margin = 1;
    let field_height = 3;
    let total_fields = input_widgets.len();
    let available_height = area.height.saturating_sub(margin * 2);
    let max_visible_fields = ((available_height / field_height) as usize)
        .max(1)
        .min(total_fields);
    let scroll_offset = app
        .current_recurring_field
        .saturating_sub(max_visible_fields - 1)
        .min(total_fields - max_visible_fields);

    let mut constraints = Vec::with_capacity(max_visible_fields + 1);
    for _ in 0..max_visible_fields {
        constraints.push(Constraint::Length(field_height));
    }
    constraints.push(Constraint::Min(0)); // Remaining space

    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(constraints)
        .split(area);

    // Render only the visible input widgets
    for (idx, widget) in input_widgets
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_visible_fields)
    {
        let chunk_index = idx - scroll_offset;
        f.render_widget(widget.clone(), form_chunks[chunk_index]);
    }

    // Main form block
    let form_block = Block::default()
        .title("Recurring Transaction Settings")
        .borders(Borders::ALL);
    f.render_widget(form_block, area);

    // Set cursor position for the end date field (field 2), adjusting for scrolling
    if app.current_recurring_field == 2 {
        let field_idx = app.current_recurring_field;
        let text_len = app.recurring_settings_fields[field_idx].len() as u16;
        if field_idx >= scroll_offset && field_idx < scroll_offset + max_visible_fields {
            let visible_idx = field_idx - scroll_offset;
            if let Some(chunk) = form_chunks.get(visible_idx) {
                f.set_cursor_position(ratatui::layout::Position::new(
                    chunk.x + text_len + 1,
                    chunk.y + 1,
                ));
            }
        }
    }
}
