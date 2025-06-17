use crate::app::state::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_filter_input(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.input_field_content.as_str())
        .style(Style::default().fg(Color::LightYellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filter (Description)"),
        );
    f.render_widget(input, area);
    // Cursor setting is handled in the main `ui` function
}

pub fn render_advanced_filter_form(f: &mut Frame, app: &App, area: Rect) {
    let field_definitions = [
        (
            "Date From (YYYY-MM-DD)",
            "(◀/▶ or +/- days, Shift+◀/▶ months (jumps to today if empty), Digits to enter)",
        ),
        (
            "Date To (YYYY-MM-DD)",
            "(◀/▶ or +/- days, Shift+◀/▶ months (jumps to today if empty), Digits to enter)",
        ),
        ("Description", ""),
        ("Category", "(Enter to select)"),
        ("Subcategory", "(Enter to select)"),
        ("Type", "(◀/▶ or Enter to toggle)"),
        ("Amount From", ""),
        ("Amount To", ""),
    ];
    let widgets: Vec<_> = app
        .advanced_filter_fields
        .iter()
        .zip(field_definitions.iter())
        .enumerate()
        .map(|(i, (text, (title, hint)))| {
            let focused = app.current_advanced_filter_field == i;
            let label = format!("{} {}", title, hint).trim_end().to_string();
            let content = if i == 5 {
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
                        .title(label)
                        .border_style(if focused {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        }),
                )
        })
        .collect();
    let margin = 1;
    let fh = 3;
    let total = widgets.len();
    let avail = area.height.saturating_sub(margin * 2);
    let maxv = ((avail / fh) as usize).max(1).min(total);
    let offset = app
        .current_advanced_filter_field
        .saturating_sub(maxv - 1)
        .min(total - maxv);
    let mut cons = vec![Constraint::Length(fh); maxv];
    cons.push(Constraint::Min(0));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(cons)
        .split(area);
    for (idx, w) in widgets.iter().enumerate().skip(offset).take(maxv) {
        f.render_widget(w.clone(), chunks[idx - offset]);
    }
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title("Advanced Filters"),
        area,
    );
    if ![3, 4, 5].contains(&app.current_advanced_filter_field) {
        let field_idx = app.current_advanced_filter_field;
        let len = app.advanced_filter_fields[field_idx].len() as u16;
        if field_idx >= offset && field_idx < offset + maxv {
            let vis = field_idx - offset;
            let ch = chunks[vis];
            f.set_cursor_position(Position::new(ch.x + len + 1, ch.y + 1));
        }
    }
}
