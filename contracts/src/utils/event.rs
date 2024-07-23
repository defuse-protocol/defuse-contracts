use near_sdk::{env, serde::Serialize, serde_json};

#[derive(Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Nep297Event<E> {
    pub standard: &'static str,
    pub version: &'static str,
    #[serde(flatten)]
    pub event: E,
}

pub trait Event<'a>: Serialize + Into<Nep297Event<Self>> {
    fn emit(self) {
        env::log_str(&format!(
            "EVENT_JSON:{}",
            serde_json::to_string(&Into::<Nep297Event<Self>>::into(self))
                .unwrap_or_else(|_| env::abort())
        ));
    }
}
impl<'a, T> Event<'a> for T where T: Serialize + Into<Nep297Event<Self>> {}
