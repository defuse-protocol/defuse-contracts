use std::collections::HashMap;

use defuse_contracts::defuse::{
    fees::Fees,
    intents::{DefuseIntents, Intent},
    tokens::TokenAmounts,
    DefuseError, Result,
};
use near_sdk::AccountId;

use crate::{
    accounts::{Account, Accounts},
    tokens::TokensBalances,
};

pub trait IntentExecutor<T> {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: T,
    ) -> Result<()>;
}

pub struct Runtime<'a> {
    pub fees: &'a Fees,
    pub total_supplies: &'a mut TokensBalances,

    pub postponed_deposits: HashMap<AccountId, TokenAmounts<u128>>,
    // TODO: bigint
    pub total_supply_deltas: TokenAmounts<i128>,
}

impl<'a> Runtime<'a> {
    #[inline]
    pub fn new(fees: &'a Fees, total_supplies: &'a mut TokensBalances) -> Self {
        Self {
            fees,
            total_supplies,
            postponed_deposits: Default::default(),
            total_supply_deltas: Default::default(),
        }
    }

    // TODO: simulate

    #[inline]
    pub fn finalize(self, accounts: &mut Accounts) -> Result<()> {
        for (receiver_id, tokens) in self.postponed_deposits {
            let receiver = accounts.get_or_create(receiver_id);
            for (token_id, amount) in tokens {
                receiver.token_balances.deposit(token_id, amount)?;
            }
        }

        if !self.total_supply_deltas.is_empty() {
            return Err(DefuseError::InvariantViolated);
        }
        Ok(())
    }
}

impl<'a> IntentExecutor<DefuseIntents> for Runtime<'a> {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: DefuseIntents,
    ) -> Result<()> {
        intent
            .intents
            .into_iter()
            .try_for_each(|intent| self.execute_intent(account_id, account, intent))
    }
}

impl<'a> IntentExecutor<Intent> for Runtime<'a> {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: Intent,
    ) -> Result<()> {
        match intent {
            Intent::AddPublicKey { public_key } => {
                account.add_public_key(account_id, public_key);
                Ok(())
            }
            Intent::RemovePublicKey { public_key } => {
                account.remove_public_key(account_id, &public_key);
                Ok(())
            }
            Intent::InvalidateNonces { nonces } => {
                for n in nonces {
                    let _ = account.commit_nonce(n);
                }
                Ok(())
            }
            Intent::TokenTransfer(intent) => self.execute_intent(account_id, account, intent),
            Intent::TokensDiff(intent) => self.execute_intent(account_id, account, intent),
            Intent::TokenWithdraw(intent) => self.execute_intent(account_id, account, intent),
        }
    }
}
