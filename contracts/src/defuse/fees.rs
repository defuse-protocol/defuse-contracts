use bnum::{cast::As, BUint};
use near_plugins::AccessControllable;
use near_sdk::{ext_contract, near, AccountId};

#[ext_contract(ext_fees_manager)]
pub trait FeesManager: AccessControllable {
    /// Set fees for both token_in and token_out
    fn set_fees(&mut self, fees: Fees);
    fn fees(&self) -> &Fees;
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Fees {
    /// Expressed in pips, i.e. 1/100th of bip, i.e. 0.0001%
    #[serde(default)]
    pub fee: u32,

    pub collector: AccountId,
}

impl Fees {
    pub const FEE_DENOMINATOR: u32 = 1_000_000;

    pub fn apply(&self, amount: u128) -> u128 {
        type U256 = BUint<4>;

        (amount.as_::<U256>() * self.fee.as_::<U256>() / Self::FEE_DENOMINATOR.as_::<U256>()).as_()
    }
}
