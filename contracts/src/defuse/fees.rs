use std::num::NonZeroU32;

use near_plugins::AccessControllable;
use near_sdk::{ext_contract, near, AccountId};

#[ext_contract(ext_fees_manager)]
pub trait FeesManager: AccessControllable {
    /// Set fees for both token_in and token_out
    fn set_fees(&mut self, fees: Fees);
    fn fees(&self) -> &Fees;
}

pub const FEE_DENOMINATOR: u32 = 1_000_000;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, Default)]
pub struct Fees {
    /// Expressed in pips, i.e. 1/100th of bip, i.e. 0.0001%
    #[serde(default)]
    pub fee: u32,

    #[serde(default)]
    pub referral_shares: u32,

    #[serde(default)]
    pub governance: Option<(
        // shares
        NonZeroU32,
        // collector
        AccountId,
    )>,
}

impl Fees {
    pub fn ref_gov(&self, amount: u128) -> (u128, u128) {
        (0, 0)
        //
        // // TODO: overflows
        // amount
        // // TODO: muldiv https://docs.uniswap.org/contracts/v3/reference/core/libraries/FullMath#muldiv
        // .checked_mul(self.fee as u128)
        // // TODO
        // .unwrap()
        // .div_ceil(FEE_DENOMINATOR as u128);
    }
}
