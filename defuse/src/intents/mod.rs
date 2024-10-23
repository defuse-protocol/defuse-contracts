mod account;
mod relayer;
mod token_diff;
mod tokens;

use defuse_contracts::{
    crypto::{Payload, SignedPayload},
    defuse::{
        events::DefuseIntentEmit,
        intents::{AccountEvent, DefuseIntents, Intent, IntentExecutedEvent, IntentsExecutor},
        payload::{DefuseMessage, MultiStandardPayload, ValidatePayloadAs},
        DefuseError, Result,
    },
    nep413::Nep413Payload,
    utils::{cache::CURRENT_ACCOUNT_ID, UnwrapOrPanic},
};
use near_plugins::{pause, Pausable};
use near_sdk::{near, serde_json, AccountId};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

#[near]
impl IntentsExecutor for DefuseImpl {
    #[pause(name = "intents")]
    #[inline]
    fn execute_intents(&mut self, intents: Vec<SignedPayload<MultiStandardPayload>>) {
        self.execute_signed_intents(intents).unwrap_or_panic()
    }

    #[handle_result]
    fn simulate_intents(
        self,
        #[allow(unused_variables)] intents: Vec<DefuseMessage<DefuseIntents>>,
    ) {
        todo!()
        // WARNING: this turns out to modify the state!!!

        // for message in intents {
        //     // make sure message is still valid
        //     if message.deadline.has_expired() {
        //         return Err(DefuseError::DeadlineExpired);
        //     }

        //     // get the account of the signer, create if doesn't exist
        //     let signer = self.accounts.get_or_create(message.signer_id.clone());

        //     // execute intent
        //     self.state
        //         .execute_intent(&message.signer_id, signer, message.message)?;
        // }
        // Ok(())
    }
}

impl DefuseImpl {
    #[inline]
    fn execute_signed_intents(
        &mut self,
        signed: impl IntoIterator<Item = SignedPayload<MultiStandardPayload>>,
    ) -> Result<()> {
        signed
            .into_iter()
            .map(|signed| self.execute_signed_intent(signed))
            .collect::<Result<Vec<_>>>()
            .map(|events| events.emit())
    }

    fn execute_signed_intent(
        &mut self,
        signed: SignedPayload<MultiStandardPayload>,
    ) -> Result<AccountEvent<'static, IntentExecutedEvent>> {
        // calculate intent hash
        let hash = signed.payload.hash();

        // verify signature of the hash
        let public_key = signed
            .signature
            .verify(&hash)
            .ok_or(DefuseError::InvalidSignature)?;

        // extract NEP-413 payload
        let Nep413Payload {
            message,
            nonce,
            recipient,
            callback_url: _,
        } = signed.payload.validate_as()?;

        // check recipient
        if recipient != *CURRENT_ACCOUNT_ID {
            return Err(DefuseError::WrongRecipient);
        }

        // deserialize message
        let DefuseMessage::<DefuseIntents> {
            signer_id,
            deadline,
            message: intents,
        } = serde_json::from_str(&message)?;

        // make sure message is still valid
        if deadline.has_expired() {
            return Err(DefuseError::DeadlineExpired);
        }

        // get the account of the signer, create if doesn't exist
        let signer = self.accounts.get_or_create(signer_id.clone());

        // make sure the account has this public key
        if !signer.has_public_key(&signer_id, &public_key) {
            return Err(DefuseError::PublicKeyNotExist);
        }

        // commit nonce
        signer.commit_nonce(nonce)?;

        // execute intent
        self.state.execute_intent(&signer_id, signer, intents)?;
        Ok(AccountEvent::new(signer_id, IntentExecutedEvent { hash }))
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

impl IntentExecutor<DefuseIntents> for State {
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
            Intent::FtWithdraw(intent) => self.execute_intent(account_id, account, intent),
            Intent::NftWithdraw(intent) => self.execute_intent(account_id, account, intent),
            Intent::MtWithdraw(intent) => self.execute_intent(account_id, account, intent),
            Intent::TokenDiff(intent) => self.execute_intent(account_id, account, intent),
        }
    }
}
