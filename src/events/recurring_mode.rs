use crate::app::fields::RecurringField;
use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_recurring_mode(app: &mut App, key_event: KeyEvent) {
    match (key_event.modifiers, key_event.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => app.exit_recurring_settings(true),
        (KeyModifiers::NONE, KeyCode::Enter) => match app.recurring_settings_fields.focused() {
            RecurringField::Frequency => app.start_frequency_selection(),
            RecurringField::IsRecurring | RecurringField::EndDate => app.save_recurring_settings(),
        },
        (KeyModifiers::NONE, KeyCode::Tab) | (KeyModifiers::NONE, KeyCode::Down) => {
            app.next_recurring_field()
        }
        (KeyModifiers::NONE, KeyCode::BackTab) | (KeyModifiers::NONE, KeyCode::Up) => {
            app.previous_recurring_field()
        }
        (KeyModifiers::NONE, KeyCode::Left) => match app.recurring_settings_fields.focused() {
            RecurringField::IsRecurring => app.toggle_recurring_enabled(),
            RecurringField::EndDate => app.decrement_date_recurring(),
            RecurringField::Frequency => {}
        },
        (KeyModifiers::NONE, KeyCode::Right) => match app.recurring_settings_fields.focused() {
            RecurringField::IsRecurring => app.toggle_recurring_enabled(),
            RecurringField::EndDate => app.increment_date_recurring(),
            RecurringField::Frequency => {}
        },
        (KeyModifiers::SHIFT, KeyCode::Left)
            if app.recurring_settings_fields.focused() == RecurringField::EndDate =>
        {
            app.decrement_month_recurring();
        }
        (KeyModifiers::SHIFT, KeyCode::Right)
            if app.recurring_settings_fields.focused() == RecurringField::EndDate =>
        {
            app.increment_month_recurring();
        }
        (KeyModifiers::NONE, KeyCode::Char(c))
            if app.recurring_settings_fields.focused() == RecurringField::EndDate =>
        {
            match c {
                '+' | '=' => app.increment_date_recurring(),
                '-' => app.decrement_date_recurring(),
                _ if c.is_ascii_digit() => app.insert_char_recurring(c),
                _ => {}
            }
        }
        (KeyModifiers::NONE, KeyCode::Backspace)
            if app.recurring_settings_fields.focused() == RecurringField::EndDate =>
        {
            app.delete_char_recurring();
        }
        _ => {}
    }
}
