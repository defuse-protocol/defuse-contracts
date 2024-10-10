use defuse_contracts::{defuse::fees::FeesManager, utils::fees::Pips};
use near_plugins::{access_control_any, pause, AccessControllable, Pausable};
use near_sdk::{near, AccountId};

use crate::{DefuseImpl, DefuseImplExt, Role};

#[near]
impl FeesManager for DefuseImpl {
    #[pause(name = "intents")]
    #[access_control_any(roles(Role::FeesManager))]
    fn set_fee(&mut self, fee: Pips) {
        self.fee = fee;
        // TODO: emit log
    }

    fn fee(&self) -> Pips {
        self.fee
    }

    #[pause(name = "intents")]
    #[access_control_any(roles(Role::FeesManager))]
    fn set_fee_collector(&mut self, fee_collector: AccountId) {
        self.fee_collector = fee_collector;
        // TODO: emit log
    }

    fn fee_collector(&self) -> &AccountId {
        &self.fee_collector
    }
}
