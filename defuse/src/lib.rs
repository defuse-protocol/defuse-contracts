mod accounts;
mod diff;
mod tokens;

use accounts::Accounts;
use defuse_contracts::defuse::Defuse;
use near_sdk::{json_types::U128, near, AccountId, BorshStorageKey, PanicOnDefault};

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct DefuseImpl {
    accounts: Accounts,
}

#[near]
impl DefuseImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            accounts: Accounts::new(Prefix::Accounts),
        }
    }
}

#[near]
impl Defuse for DefuseImpl {
    #[inline]
    fn mt_balance_of(&self, account_id: &AccountId, token_id: &String) -> U128 {
        let token_id = token_id.parse().unwrap();
        U128(
            self.accounts
                .get(account_id)
                .map(|account| account.token_balances.balance_of(&token_id))
                .unwrap_or_default(),
        )
    }

    #[inline]
    fn mt_batch_balance_of(&self, account_id: &AccountId, token_ids: &Vec<String>) -> Vec<U128> {
        token_ids
            .iter()
            .map(|token_id| self.mt_balance_of(account_id, token_id))
            .collect()
    }
}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
enum Prefix {
    Accounts,
}
