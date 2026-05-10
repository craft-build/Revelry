use serde::{Deserialize, Serialize};

/// Application setting key-value pair stored in the app_settings table.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
}
