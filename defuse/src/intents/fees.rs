use defuse_contracts::defuse::fees::{Fees, FeesManager};
use near_plugins::{access_control_any, AccessControllable};
use near_sdk::near;

use crate::{DefuseImpl, DefuseImplExt, Role};

#[near]
impl FeesManager for DefuseImpl {
    #[access_control_any(roles(Role::FeesManager))]
    fn set_fees(&mut self, fees: Fees) {
        self.fees = fees;
    }

    fn fees(&self) -> &Fees {
        &self.fees
    }
}
