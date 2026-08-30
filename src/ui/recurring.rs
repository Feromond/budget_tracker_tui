use crate::app::fields::{FieldKey, FieldKind};
use crate::app::state::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

pub fn render_recurring_settings(f: &mut Frame, app: &App, area: Rect) {
    let focused_field = app.recurring_settings_fields.focused();
    let input_widgets: Vec<_> = app
        .recurring_settings_fields
        .iter()
        .map(|(field, text)| {
            let is_focused = field == focused_field;
            let title = format!("{} {}", field.label(), field.hint())
                .trim_end()
                .to_string();

            let content = match field.kind() {
                FieldKind::Toggle => Span::styled(
                    format!(" < {} > ", text),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                FieldKind::Selection => Span::styled(
                    format!("  {}  ", text),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                _ if text.is_empty() => Span::styled(
                    " (Optional - leave empty for no end date) ",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
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
    let scroll_offset = focused_field
        .index()
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

    if focused_field.kind().is_editable() {
        let field_idx = focused_field.index();
        let text_len = app.recurring_settings_fields[focused_field].len() as u16;
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
