use defuse_contracts::{
    defuse::{
        events::DefuseIntentEmit,
        fees::{FeeChangedEvent, FeeCollectorChangedEvent, FeesManager},
    },
    utils::fees::Pips,
};
use near_plugins::{access_control_any, pause, AccessControllable, Pausable};
use near_sdk::{near, require, AccountId};

use crate::{DefuseImpl, DefuseImplExt, Role};

#[near]
impl FeesManager for DefuseImpl {
    #[pause(name = "intents")]
    #[access_control_any(roles(Role::FeesManager))]
    fn set_fee(&mut self, fee: Pips) {
        require!(self.fee != fee, "same");

        FeeChangedEvent {
            old_fee: &self.fee,
            new_fee: &fee,
        }
        .emit();

        self.fee = fee;
    }

    fn fee(&self) -> Pips {
        self.fee
    }

    #[pause(name = "intents")]
    #[access_control_any(roles(Role::FeesManager))]
    fn set_fee_collector(&mut self, fee_collector: AccountId) {
        require!(self.fee_collector != fee_collector, "same");

        FeeCollectorChangedEvent {
            old_fee_collector: &self.fee_collector,
            new_fee_collector: &fee_collector,
        }
        .emit();

        self.fee_collector = fee_collector;
    }

    fn fee_collector(&self) -> &AccountId {
        &self.fee_collector
    }
}
