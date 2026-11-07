use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Input {
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub tool_response: Value,
}

impl Input {
    pub fn str_field(&self, k: &str) -> Option<&str> {
        self.tool_input.get(k)?.as_str()
    }
}
