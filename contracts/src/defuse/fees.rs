use std::borrow::Cow;

use near_plugins::AccessControllable;
use near_sdk::{ext_contract, near, AccountId, AccountIdRef};

use crate::utils::fees::Pips;

#[ext_contract(ext_fees_manager)]
pub trait FeesManager: AccessControllable {
    /// Set fees for both token_in and token_out
    fn set_fee(&mut self, fee: Pips);
    fn fee(&self) -> Pips;

    fn set_fee_collector(&mut self, fee_collector: AccountId);
    fn fee_collector(&self) -> &AccountId;
}

#[must_use = "make sure to `.emit()` this event"]
#[near(serializers = [json])]
#[derive(Debug)]
pub struct FeeChangedEvent {
    pub old_fee: Pips,
    pub new_fee: Pips,
}

#[must_use = "make sure to `.emit()` this event"]
#[near(serializers = [json])]
#[derive(Debug)]
pub struct FeeCollectorChangedEvent<'a> {
    #[serde(borrow)]
    pub old_fee_collector: Cow<'a, AccountIdRef>,
    #[serde(borrow)]
    pub new_fee_collector: Cow<'a, AccountIdRef>,
}
