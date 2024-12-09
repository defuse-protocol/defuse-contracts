use near_sdk::{env, Gas};

#[inline]
pub fn gas_left() -> Gas {
    env::prepaid_gas().saturating_sub(env::used_gas())
}
