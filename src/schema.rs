use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Topic {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

impl Topic {
    pub fn new(title: String, description: String) -> Self {
        Topic { title, description }
    }
}
