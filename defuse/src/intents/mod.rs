mod token_diff;
mod tokens;

use defuse_contracts::defuse::{
    intents::{DefuseIntents, Intent, SignedIntentExecutor},
    message::SignedDefuseMessage,
    Result,
};
use near_sdk::{near, AccountId};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

#[near]
impl SignedIntentExecutor for DefuseImpl {
    #[handle_result]
    fn execute_signed_intents(
        &mut self,
        signed: Vec<SignedDefuseMessage<DefuseIntents>>,
    ) -> Result<()> {
        #[cfg(feature = "beta")]
        crate::beta::beta_access!(self);

        for signed in signed {
            let (signer_id, signer, intents) = self.accounts.verify_signed_message(signed)?;

            for intent in intents.intents {
                self.state.execute_intent(&signer_id, signer, intent)?;
            }
            // TODO: log intent hash?
        }

        Ok(())
    }
}

pub trait IntentExecutor<T> {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: T,
    ) -> Result<()>;
}

impl IntentExecutor<Intent> for State {
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
            Intent::TokenTransferCall(intent) => self.execute_intent(account_id, account, intent),

            Intent::TokensDiff(intent) => self.execute_intent(account_id, account, intent),

            Intent::FtWithdraw(intent) => self.execute_intent(account_id, account, intent),
            Intent::NftWithdraw(intent) => self.execute_intent(account_id, account, intent),
            Intent::MtWithdraw(intent) => self.execute_intent(account_id, account, intent),
        }
    }
}
