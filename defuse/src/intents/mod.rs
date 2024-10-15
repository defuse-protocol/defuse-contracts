mod account;
mod relayer;
mod token_diff;
mod tokens;

use defuse_contracts::{
    crypto::Payload,
    defuse::{
        events::DefuseIntentEmit,
        intents::{DefuseIntents, Intent, IntentExecutedEvent, IntentsExecutor},
        payload::{DefuseMessage, SignedDefuseMessage, ValidatePayloadAs},
        DefuseError, Result,
    },
    nep413::Nep413Payload,
};
use near_plugins::{pause, Pausable};
use near_sdk::{near, AccountId};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

#[near]
impl IntentsExecutor for DefuseImpl {
    #[pause(name = "intents")]
    #[handle_result]
    fn execute_intents(&mut self, intents: Vec<SignedDefuseMessage<DefuseIntents>>) -> Result<()> {
        for signed in intents {
            // calculate intent hash
            let hash = signed.payload.hash();

            // verify signature of the hash
            let public_key = signed
                .signature
                .verify(&hash)
                .ok_or(DefuseError::InvalidSignature)?;

            // extract NEP-413 payload
            let payload: Nep413Payload<_> = signed.payload.validate_as()?;

            // signer_id is encoded in the signed message
            let signer_id = &payload.message.signer_id;

            // get the account of the signer, create if doesn't exist
            let signer = self.accounts.get_or_create(signer_id.clone());

            // make sure the account has this public key
            if !signer.has_public_key(signer_id, &public_key) {
                return Err(DefuseError::InvalidSignature);
            }

            // verify NEP-413 payload
            let DefuseMessage {
                signer_id,
                deadline,
                message: intent,
            } = signer.verify_nep413_payload(payload)?;

            // make message is still valid
            if deadline.has_expired() {
                return Err(DefuseError::DeadlineExpired);
            }

            // execute intent
            self.state.execute_intent(&signer_id, signer, intent)?;
            IntentExecutedEvent { hash: &hash }.emit();
        }
        Ok(())
    }

    #[handle_result]
    fn simulate_intents(mut self, intents: Vec<DefuseMessage<DefuseIntents>>) -> Result<()> {
        for message in intents {
            // make message is still valid
            if message.deadline.has_expired() {
                return Err(DefuseError::DeadlineExpired);
            }

            // get the account of the signer, create if doesn't exist
            let signer = self.accounts.get_or_create(message.signer_id.clone());

            // execute intent
            self.state
                .execute_intent(&message.signer_id, signer, message.message)?;
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

impl<T> IntentExecutor<DefuseIntents> for T
where
    T: IntentExecutor<Intent>,
{
    #[inline]
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: DefuseIntents,
    ) -> Result<()> {
        for intent in intent.intents {
            self.execute_intent(account_id, account, intent)?;
        }

        Ok(())
    }
}

impl IntentExecutor<Intent> for State {
    #[inline]
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: Intent,
    ) -> Result<()> {
        match intent {
            Intent::AddPublicKey(intent) => self.execute_intent(account_id, account, intent),
            Intent::RemovePublicKey(intent) => self.execute_intent(account_id, account, intent),
            Intent::InvalidateNonces(intent) => self.execute_intent(account_id, account, intent),
            Intent::MtBatchTransfer(intent) => self.execute_intent(account_id, account, intent),
            Intent::MtBatchTransferCall(intent) => self.execute_intent(account_id, account, intent),
            Intent::TokenDiff(intent) => self.execute_intent(account_id, account, intent),
            Intent::FtWithdraw(intent) => self.execute_intent(account_id, account, intent),
            Intent::NftWithdraw(intent) => self.execute_intent(account_id, account, intent),
            Intent::MtWithdraw(intent) => self.execute_intent(account_id, account, intent),
        }
    }
}
