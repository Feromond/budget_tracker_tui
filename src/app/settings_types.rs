
#[derive(Debug, Clone, PartialEq)]
pub enum SettingType {
    SectionHeader,
    Path,
    Number,
    Toggle,
}

#[derive(Debug, Clone)]
pub struct SettingItem {
    pub key: String,
    pub label: String,
    pub value: String,
    pub setting_type: SettingType,
    pub help: String,
}


#[derive(Debug, Default)]
pub struct SettingsState {
    pub items: Vec<SettingItem>,
    pub selected_index: usize,
    // For text editing:
    pub edit_cursor: usize, 
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            edit_cursor: 0,
        }
    }

    pub fn add_setting(
        &mut self, 
        key: &str, 
        label: &str, 
        value: String, 
        setting_type: SettingType, 
        help: &str
    ) {
        self.items.push(SettingItem {
            key: key.to_string(),
            label: label.to_string(),
            value,
            setting_type,
            help: help.to_string(),
        });
    }
    
    pub fn get_value(&self, key: &str) -> Option<&String> {
        self.items.iter().find(|item| item.key == key).map(|item| &item.value)
    }
}
