use near_sdk::{env, serde::Serialize, serde_json};

pub trait JsonLog: Serialize {
    fn log_json(&self) -> Result<(), serde_json::Error> {
        env::log_str(&serde_json::to_string(self)?);
        Ok(())
    }
}
impl<T> JsonLog for T where T: Serialize {}
