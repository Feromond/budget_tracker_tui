use crate::app::state::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_category_catalog(f: &mut Frame, app: &mut App, area: Rect) {
    let title = format!(
        " Category Catalog ({}) ",
        app.database_path.to_string_lossy()
    );

    if app.category_records.is_empty() {
        let empty = Paragraph::new("No categories found. Press 'a' to add one.")
            .block(Block::default().title(title).borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(["Type", "Category", "Subcategory", "Tag", "Target Budget"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let rows = app.category_records.iter().map(|record| {
        Row::new(vec![
            Cell::from(record.transaction_type.to_string()),
            Cell::from(record.category.clone()),
            Cell::from(if record.subcategory.is_empty() {
                "(None)".to_string()
            } else {
                record.subcategory.clone()
            }),
            Cell::from(record.tag.clone().unwrap_or_default()),
            Cell::from(
                Line::from(
                    record
                        .target_budget
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                )
                .alignment(Alignment::Right),
            ),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Percentage(30),
            Constraint::Percentage(26),
            Constraint::Percentage(18),
            Constraint::Percentage(16),
        ],
    )
    .header(header)
    .block(Block::default().title(title).borders(Borders::ALL))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut app.category_table_state);
}

pub fn render_category_editor(f: &mut Frame, app: &App, area: Rect) {
    let field_definitions = [
        ("Transaction Type", "(Left/Right or Enter to toggle)"),
        ("Category", ""),
        ("Subcategory", "(Optional)"),
        ("Tag", "(Optional)"),
        ("Target Budget", "(Optional, positive number)"),
    ];
    let input_widgets: Vec<_> = app
        .category_edit_fields
        .iter()
        .zip(field_definitions.iter())
        .enumerate()
        .map(|(index, (text, (base_title, hint)))| {
            let is_focused = app.current_category_field == index;
            let title = format!("{} {}", base_title, hint).trim_end().to_string();
            let content = if index == 0 {
                Span::styled(
                    format!(" < {} > ", text),
                    Style::default().fg(Color::White).bold(),
                )
            } else {
                Span::raw(text.as_str())
            };

            Paragraph::new(content).block(
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

    let margin = 1;
    let field_height = 3;
    let total_fields = input_widgets.len();
    let available_height = area.height.saturating_sub(margin * 2);
    let max_visible_fields = ((available_height / field_height) as usize)
        .max(1)
        .min(total_fields);
    let scroll_offset = app
        .current_category_field
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

    for (index, widget) in input_widgets
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_visible_fields)
    {
        let chunk_index = index - scroll_offset;
        f.render_widget(widget.clone(), form_chunks[chunk_index]);
    }

    let form_title = if app.editing_category_id.is_some() {
        "Edit Category"
    } else {
        "Add Category"
    };
    let form_block = Block::default()
        .title(form_title)
        .title_bottom(" [Esc] Cancel, [Enter] Toggle/Save ")
        .borders(Borders::ALL);
    f.render_widget(form_block, area);

    if app.current_category_field != 0 {
        let field_idx = app.current_category_field;
        let text = &app.category_edit_fields[field_idx];
        let cursor_byte_idx = app.category_edit_cursor.min(text.len());
        let visual_cursor = text[..cursor_byte_idx].chars().count() as u16;

        if field_idx >= scroll_offset && field_idx < scroll_offset + max_visible_fields {
            let visible_idx = field_idx - scroll_offset;
            if let Some(chunk) = form_chunks.get(visible_idx) {
                f.set_cursor_position(Position::new(chunk.x + visual_cursor + 1, chunk.y + 1));
            }
        }
    }
}
