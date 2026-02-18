/// Message that must be shared across logger senders.
pub struct Message {
    modules: Vec<String>,
    text: String,
}

impl Message {
    pub fn new(modules: &Vec<String>, text: &str) -> Self {
        Self {
            modules: modules.clone(),
            text: text.into(),
        }
    }

    pub fn modules(&self) -> &Vec<String> {
        &self.modules
    }

    pub fn text(&self) -> &String {
        &self.text
    }
}
