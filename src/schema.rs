use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
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
