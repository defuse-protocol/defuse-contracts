use std::collections::HashMap;

use defuse_contracts::defuse::{
    fees::{Fees, FeesManager},
    tokens::{TokenAmounts, TokenId},
    Result,
};
use near_plugins::{access_control_any, AccessControllable};
use near_sdk::{near, AccountId};

use crate::{accounts::Accounts, DefuseImpl, DefuseImplExt, Role};

use super::tokens::TokenTracker;

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

pub struct FeesTracker {
    fees: Fees,

    referrals_collected: HashMap<AccountId, TokenAmounts<u128>>,
    governance_collected: TokenAmounts<u128>,
}

impl FeesTracker {
    #[inline]
    pub fn new(fees: Fees) -> Self {
        Self {
            fees,
            referrals_collected: Default::default(),
            governance_collected: Default::default(),
        }
    }

    pub fn on_token_amount(
        &mut self,
        token_id: TokenId,
        amount: u128,
        referral: Option<AccountId>,
    ) -> Result<()> {
        let (referral_fee, governance_fee) = self.fees.ref_gov(amount);
        self.governance_collected
            .add(token_id.clone(), governance_fee)?;

        if referral_fee == 0 {
            return Ok(());
        }
        let Some(referral) = referral else {
            return Ok(());
        };
        self.referrals_collected
            .entry(referral)
            .or_default()
            .add(token_id, referral_fee)?;

        Ok(())
    }

    pub fn finalize(self, accounts: &mut Accounts, token_tracker: &mut TokenTracker) -> Result<()> {
        if let Some((_shares, gov_collector)) = self.fees.governance {
            let gov_collector = accounts.get_or_create(gov_collector);
            for (token_id, amount) in self.governance_collected {
                token_tracker.deposit(&mut gov_collector.token_balances, token_id, amount)?;
            }
        }

        for (referral, collected) in self.referrals_collected {
            let referral = accounts.get_or_create(referral);
            for (token_id, amount) in collected {
                token_tracker.deposit(&mut referral.token_balances, token_id, amount)?;
            }
        }

        Ok(())
    }
}
