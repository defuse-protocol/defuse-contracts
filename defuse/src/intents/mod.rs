mod account;
mod relayer;
mod token_diff;
mod tokens;

use defuse_contracts::{
    crypto::{Payload, PublicKey},
    defuse::{
        intents::{DefuseIntents, Intent, IntentsExecutor},
        payload::{DefusePayload, SignedDefusePayload, SignerPayload, ValidatePayloadAs},
        DefuseError, Result,
    },
};
use near_plugins::{pause, Pausable};
use near_sdk::{borsh::BorshSerialize, near, AccountId};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

#[near]
impl IntentsExecutor for DefuseImpl {
    #[pause(name = "intents")]
    #[handle_result]
    fn execute_intents(
        &mut self,
        #[serializer(borsh)] intents: Vec<SignedDefusePayload<DefuseIntents>>,
    ) -> Result<()> {
        for signed in intents {
            self.execute_signed_intent(signed)?;
        }
        Ok(())
    }
    #[handle_result]
    fn execute_intents_json(
        &mut self,
        intents: Vec<SignedDefusePayload<DefuseIntents>>,
    ) -> Result<()> {
        self.execute_intents(intents)
    }
    #[pause(name = "intents")]
    #[handle_result]
    fn simulate_intents(
        mut self,
        #[serializer(borsh)] intents: Vec<DefusePayload<DefuseIntents>>,
    ) -> Result<()> {
        for payload in intents {
            self.execute_payload_intent(payload, None)?;
        }
        Ok(())
    }

    #[handle_result]
    fn simulate_intents_json(self, intents: Vec<DefusePayload<DefuseIntents>>) -> Result<()> {
        self.simulate_intents(intents)
    }
}

impl DefuseImpl {
    pub fn execute_signed_intent<T>(&mut self, signed: SignedDefusePayload<T>) -> Result<()>
    where
        T: BorshSerialize,
        State: IntentExecutor<T>,
    {
        // calculate intent hash
        let intent_hash = signed.payload.hash();

        // verify signature of the hash
        let public_key = signed
            .signature
            .verify(&intent_hash)
            .ok_or(DefuseError::InvalidSignature)?;

        // extract NEP-413 payload
        let payload: DefusePayload<_> = signed.payload.validate_as()?;

        self.execute_payload_intent(payload, public_key)?;

        // TODO: log hash

        Ok(())
    }

    fn execute_payload_intent<T>(
        &mut self,
        payload: DefusePayload<T>,
        verify_public_key: impl Into<Option<PublicKey>>,
    ) -> Result<()>
    where
        State: IntentExecutor<T>,
    {
        // signer_id is encoded in the signed message
        let signer_id = &payload.message.signer_id;

        // get the account of the signer, create if doesn't exist
        let signer = self.accounts.get_or_create(signer_id.clone());

        if let Some(public_key) = verify_public_key.into() {
            // make sure the account has this public key
            if !signer.has_public_key(signer_id, &public_key) {
                return Err(DefuseError::InvalidSignature);
            }
        }

        // verify NEP-413 payload
        let SignerPayload {
            signer_id,
            payload: intent,
        } = signer.verify_nep413_payload(payload)?;

        // execute intent
        self.state.execute_intent(&signer_id, signer, intent)
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
        if intent.deadline.has_expired() {
            return Err(DefuseError::DeadlineExpired);
        }

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
