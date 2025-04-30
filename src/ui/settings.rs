use crate::app::state::App;
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::widgets::Clear;

pub fn render_settings_form(f: &mut Frame, app: &App, area: Rect) {
    let field_definitions = [
        ("Data File Path", ""),
        ("Target Budget", "(optional, numeric)"),
    ];
    let input_width = area.width.saturating_sub(8).max(10); // leave margin for borders
    let mut scroll_offsets = [0u16; 2];
    // For Data File Path, calculate scroll offset so cursor is always visible
    let cursor_col = app.input_field_cursor as u16;
    if cursor_col >= input_width {
        scroll_offsets[0] = cursor_col - input_width + 1;
    } else {
        scroll_offsets[0] = 0;
    }
    scroll_offsets[1] = 0;
    let input_widgets: Vec<_> = app
        .settings_fields
        .iter()
        .zip(field_definitions.iter())
        .enumerate()
        .map(|(i, (text, (base_title, hint)))| {
            let is_focused = app.current_settings_field == i;
            let title = format!("{} {}", base_title, hint).trim_end().to_string();
            if i == 0 {
                Paragraph::new(text.as_str())
                    .style(Style::default().fg(Color::White))
                    .scroll((0, scroll_offsets[0]))
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
            } else {
                Paragraph::new(text.as_str())
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
            }
        })
        .collect();

    let margin = 2;
    let field_height = 3;
    let total_fields = input_widgets.len();
    let available_height = area.height.saturating_sub(margin * 2);
    let max_visible_fields = ((available_height / field_height) as usize)
        .max(1)
        .min(total_fields);
    let scroll_offset = app
        .current_settings_field
        .saturating_sub(max_visible_fields - 1)
        .min(total_fields - max_visible_fields);

    let mut constraints = Vec::with_capacity(max_visible_fields + 1);
    for _ in 0..max_visible_fields {
        constraints.push(Constraint::Length(field_height));
    }
    constraints.push(Constraint::Min(0));
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(constraints)
        .split(area);

    f.render_widget(Clear, area);
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default().bg(Color::Black))
        .title("Settings");
    f.render_widget(popup_block, area);

    for (idx, widget) in input_widgets
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_visible_fields)
    {
        let chunk_index = idx - scroll_offset;
        f.render_widget(widget.clone(), form_chunks[chunk_index]);
    }

    // Set cursor position for editable text fields, adjusting for scrolling
    let field_idx = app.current_settings_field;
    let visible_cursor_x = if field_idx == 0 {
        let cursor_col = app.input_field_cursor as u16;
        let scroll_offset = scroll_offsets[0];
        cursor_col.saturating_sub(scroll_offset)
    } else {
        app.settings_fields[field_idx].len() as u16
    };
    if field_idx >= scroll_offset && field_idx < scroll_offset + max_visible_fields {
        let visible_idx = field_idx - scroll_offset;
        if let Some(chunk) = form_chunks.get(visible_idx) {
            f.set_cursor_position(Position::new(chunk.x + visible_cursor_x + 1, chunk.y + 1));
        }
    }
} 