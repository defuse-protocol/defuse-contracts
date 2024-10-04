use near_sdk::{near, AccountId};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl DefuseImpl {
    pub fn grant_beta_access(&mut self, user: AccountId) {}
}

macro_rules! beta_access {
    ($contract:expr $(, $account_id:expr)*) => {
        ::near_sdk::require!(
            [
                ::defuse_contracts::utils::cache::PREDECESSOR_ACCOUNT_ID.clone(),
                $($account_id, )?
            ]
            .into_iter()
            .all(|account| ::near_plugins::AccessControllable::acl_has_role(
                $contract,
                crate::Role::BetaAccess.into(),
                account)
            ),
            "closed beta"
        );
    };
}
pub(crate) use beta_access;
