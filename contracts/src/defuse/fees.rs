use near_plugins::AccessControllable;
use near_sdk::{ext_contract, serde::Serialize, AccountId};

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
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct FeeChangedEvent<'a> {
    pub old_fee: &'a Pips,
    pub new_fee: &'a Pips,
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct FeeCollectorChangedEvent<'a> {
    pub old_fee_collector: &'a AccountId,
    pub new_fee_collector: &'a AccountId,
}
