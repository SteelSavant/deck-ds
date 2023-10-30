use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStartSettings {
    pub app_id: String,
}
