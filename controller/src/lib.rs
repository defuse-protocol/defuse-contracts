use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId, PanicOnDefault};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct ControllerContract {
    owner_id: AccountId,
}

#[near_bindgen]
impl ControllerContract {
    #[init]
    #[must_use]
    #[allow(clippy::use_self)]
    pub const fn new(owner_id: AccountId) -> Self {
        Self { owner_id }
    }
}
