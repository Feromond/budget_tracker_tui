use crate::app::state::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_transaction_form(f: &mut Frame, app: &App, area: Rect) {
    // Field titles and hints
    let field_definitions = [
        (
            "Date (YYYY-MM-DD)",
            "(◀/▶ or +/- for days, Shift+◀/▶ for months, Digits to enter)",
        ),
        ("Description", ""),
        ("Amount", ""),
        ("Type", "(◀/▶ or Enter to toggle)"),
        ("Category", "(Enter to select)"),
        ("Subcategory", "(Enter to select)"),
    ];
    let input_widgets: Vec<_> = app
        .add_edit_fields
        .iter()
        .zip(field_definitions.iter())
        .enumerate()
        .map(|(i, (text, (base_title, hint)))| {
            let is_focused = app.current_add_edit_field == i;
            let title = format!("{} {}", base_title, hint).trim_end().to_string();
            let content = if i == 3 {
                Span::styled(
                    format!(" < {} > ", text),
                    Style::default().fg(Color::White).bold(),
                )
            } else {
                Span::raw(text.as_str())
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
        .current_add_edit_field
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

    let form_title_text = if app.mode == crate::app::state::AppMode::Editing {
        "Edit Transaction"
    } else {
        "Add New Transaction"
    };
    let form_block = Block::default()
        .title(form_title_text)
        .borders(Borders::ALL);
    f.render_widget(form_block, area);

    // Set cursor position for editable text fields, adjusting for scrolling
    if ![3, 4, 5].contains(&app.current_add_edit_field) {
        let field_idx = app.current_add_edit_field;
        let text_len = app.add_edit_fields[field_idx].len() as u16;
        if field_idx >= scroll_offset && field_idx < scroll_offset + max_visible_fields {
            let visible_idx = field_idx - scroll_offset;
            if let Some(chunk) = form_chunks.get(visible_idx) {
                f.set_cursor_position(Position::new(chunk.x + text_len + 1, chunk.y + 1));
            }
        }
    }
} 