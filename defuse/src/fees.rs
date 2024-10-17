use core::mem;
use std::borrow::Cow;

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
    fn set_fee(&mut self, #[allow(unused_mut)] mut fee: Pips) {
        require!(self.fee != fee, "same");
        mem::swap(&mut self.fee, &mut fee);
        FeeChangedEvent {
            old_fee: fee,
            new_fee: self.fee,
        }
        .emit();
    }

    fn fee(&self) -> Pips {
        self.fee
    }

    #[pause(name = "intents")]
    #[access_control_any(roles(Role::FeesManager))]
    fn set_fee_collector(&mut self, #[allow(unused_mut)] mut fee_collector: AccountId) {
        require!(self.fee_collector != fee_collector, "same");
        mem::swap(&mut self.fee_collector, &mut fee_collector);
        FeeCollectorChangedEvent {
            old_fee_collector: fee_collector.into(),
            new_fee_collector: Cow::Borrowed(self.fee_collector.as_ref()),
        }
        .emit();
    }

    fn fee_collector(&self) -> &AccountId {
        &self.fee_collector
    }
}
