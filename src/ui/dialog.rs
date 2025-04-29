use crate::app::state::App;
use crate::ui::helpers::centered_rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_confirmation_dialog(f: &mut Frame, message: &str, area: Rect) {
    let dialog_area = centered_rect(60, 20, area);

    let dialog_block = Block::default()
        .title("Confirmation")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let dialog_text = Paragraph::new(message)
        .block(dialog_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, dialog_area); // Clear the area behind the dialog
    f.render_widget(dialog_text, dialog_area);
}

pub fn render_selection_popup(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_title = match app.mode {
        crate::app::state::AppMode::SelectingCategory => "Select Category (Enter/Esc)",
        crate::app::state::AppMode::SelectingSubcategory => "Select Subcategory (Enter/Esc)",
        _ => "Select Option",
    };

    let items: Vec<ListItem> = app
        .current_selection_list
        .iter()
        .map(|i| ListItem::new(i.as_str()).style(Style::default().fg(Color::White)))
        .collect();

    let list = List::new(items)
        .block(Block::default().title(popup_title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let popup_area = centered_rect(60, 50, area);

    f.render_widget(Clear, popup_area);
    f.render_stateful_widget(list, popup_area, &mut app.selection_list_state);
}

pub fn render_settings_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(75, 30, area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(3), // Input field
            Constraint::Length(1), // Instructions
            Constraint::Min(0),
        ])
        .split(popup_area);

    let title = Paragraph::new("Settings: Data File Path")
        .alignment(Alignment::Center)
        .style(Style::default().bold());

    let input_chunk = popup_chunks[1];

    let input_width = input_chunk.width.saturating_sub(2); // Subtract 2 for borders
    let cursor_col = app.input_field_content[..app.input_field_cursor]
        .chars()
        .count() as u16;
    let scroll_offset = cursor_col.saturating_sub(input_width.saturating_sub(1)); // Scroll only when cursor nears the right edge

    let input = Paragraph::new(app.input_field_content.as_str())
        .style(Style::default().fg(Color::White))
        .scroll((0, scroll_offset))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Path")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    let instructions = Paragraph::new("Esc: Cancel, Enter: Save, Ctrl+D: Reset, Ctrl+U: Clear")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    // Render the popup background
    let clear_area = Clear;
    f.render_widget(clear_area, popup_area);
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title("Settings")
        .style(Style::default().bg(Color::Black));
    f.render_widget(popup_block, popup_area);

    // Render widgets inside the popup area
    f.render_widget(title, popup_chunks[0]);
    f.render_widget(input, input_chunk);
    f.render_widget(instructions, popup_chunks[2]);

    let visible_cursor_x = cursor_col.saturating_sub(scroll_offset);
    f.set_cursor_position(Position::new(
        input_chunk.x + visible_cursor_x + 1, // Add 1 for left border
        input_chunk.y + 1,                    // Add 1 for top border
    ));
}
