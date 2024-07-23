use near_sdk::{env, serde::Serialize, serde_json};

pub trait JsonLog: Serialize {
    fn emit(&self) {
        env::log_str(&format!(
            "EVENT_JSON:{}",
            serde_json::to_string(self).unwrap_or_else(|_| env::abort())
        ));
    }
}
impl<T> JsonLog for T where T: Serialize {}
