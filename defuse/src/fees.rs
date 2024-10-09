use defuse_contracts::{defuse::fees::FeesManager, utils::fees::Pips};
use near_sdk::{near, AccountId};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl FeesManager for DefuseImpl {
    // TODO: ACL
    // #[access_control_any(roles(Role::FeesManager))]
    fn set_fee(&mut self, fee: Pips) {
        self.fee = fee;
    }

    fn fee(&self) -> Pips {
        self.fee
    }

    // TODO: ACL
    fn set_fee_collector(&mut self, fee_collector: AccountId) {
        self.fee_collector = fee_collector;
    }

    fn fee_collector(&self) -> &AccountId {
        &self.fee_collector
    }
}
