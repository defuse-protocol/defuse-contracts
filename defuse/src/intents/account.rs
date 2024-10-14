use defuse_contracts::defuse::{
    intents::account::{AddPublicKey, InvalidateNonces, RemovePublicKey},
    Result,
};
use near_sdk::AccountId;

use crate::{accounts::Account, state::State};

use super::IntentExecutor;

impl IntentExecutor<AddPublicKey> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: AddPublicKey,
    ) -> Result<()> {
        account.add_public_key(account_id, intent.public_key)
    }
}

impl IntentExecutor<RemovePublicKey> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: RemovePublicKey,
    ) -> Result<()> {
        account.remove_public_key(account_id, &intent.public_key)
    }
}

impl IntentExecutor<InvalidateNonces> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        _account_id: &AccountId,
        account: &mut Account,
        intent: InvalidateNonces,
    ) -> Result<()> {
        for n in intent.nonces {
            account.commit_nonce(n)?;
        }
        Ok(())
    }
}
