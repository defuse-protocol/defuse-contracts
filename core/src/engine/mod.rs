mod inspector;
mod runtime;
mod state;

pub use self::{inspector::*, runtime::*, state::*};

use defuse_crypto::{Payload, SignedPayload};
use near_sdk::{AccountId, AccountIdRef};

use crate::{
    intents::{tokens::Transfer, DefuseIntents, ExecutableIntent},
    payload::{multi::MultiPayload, DefusePayload, ExtractDefusePayload},
    tokens::TokenId,
    DefuseError, Result,
};

pub struct Engine<S, I> {
    // TODO: make private
    // TODO: impl State for TransferMatcherState
    pub state: S,
    pub inspector: I,
    pub deltas: TransferMatcher,
}

impl<S, I> Engine<S, I>
where
    S: State,
    I: Inspector,
{
    #[inline]
    pub fn new(state: S, inspector: I) -> Self {
        Self {
            state,
            inspector,
            deltas: Default::default(),
        }
    }

    pub fn execute_signed_intents(
        &mut self,
        signed: impl IntoIterator<Item = MultiPayload>,
    ) -> Result<()> {
        for signed in signed {
            self.execute_signed_intent(signed)?;
        }
        Ok(())
    }

    pub fn execute_signed_intent(&mut self, signed: MultiPayload) -> Result<()> {
        // verify signed payload and get public key
        let public_key = signed.verify().ok_or(DefuseError::InvalidSignature)?;

        // calculate intent hash
        let hash = signed.hash();

        // extract NEP-413 payload
        let DefusePayload::<DefuseIntents> {
            signer_id,
            verifying_contract,
            deadline,
            nonce,
            message: intents,
        } = signed.extract_defuse_payload()?;

        // check recipient
        if verifying_contract != *self.state.verifying_contract() {
            return Err(DefuseError::WrongVerifyingContract);
        }

        self.inspector.on_deadline(deadline);
        // make sure message is still valid
        if deadline.has_expired() {
            return Err(DefuseError::DeadlineExpired);
        }

        // make sure the account has this public key
        if !self.state.has_public_key(&signer_id, &public_key) {
            return Err(DefuseError::PublicKeyNotExist);
        }

        // commit nonce
        if !self.state.commit_nonce(signer_id.clone(), nonce) {
            return Err(DefuseError::NonceUsed);
        }

        intents.execute_intent(&signer_id, self)?;
        self.inspector.on_intent_executed(&signer_id, hash);

        Ok(())
    }

    pub(crate) fn internal_deposit(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Result<()> {
        self.state
            .internal_deposit(owner_id.clone(), [(token_id.clone(), amount)])?;
        if !self.deltas.deposit(owner_id, token_id, amount) {
            return Err(DefuseError::BalanceOverflow);
        }
        Ok(())
    }
    pub(crate) fn internal_withdraw(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Result<()> {
        self.state
            .internal_withdraw(&owner_id, [(token_id.clone(), amount)])?;
        if !self.deltas.withdraw(owner_id, token_id, amount) {
            return Err(DefuseError::BalanceOverflow);
        }
        Ok(())
    }

    pub(crate) fn internal_add_delta(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        delta: i128,
    ) -> Result<()> {
        let amount = delta.unsigned_abs();
        if delta.is_negative() {
            self.internal_withdraw(owner_id, token_id, amount)?;
        } else {
            self.internal_deposit(owner_id.clone(), token_id, amount)?;
        }
        Ok(())
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountIdRef,
        transfer: Transfer,
    ) -> Result<()> {
        if sender_id == transfer.receiver_id || transfer.tokens.is_empty() {
            return Err(DefuseError::InvalidIntent);
        }
        self.inspector.on_transfer(sender_id, &transfer);
        for (token_id, amount) in transfer.tokens {
            self.internal_withdraw(sender_id.to_owned(), token_id.clone(), amount)?;
            self.internal_deposit(transfer.receiver_id.clone(), token_id, amount)?;
        }
        Ok(())
    }

    pub fn finalize(self) -> Result<Transfers> {
        self.deltas
            .finalize()
            .map_err(|unmatched_deltas| DefuseError::InvariantViolated { unmatched_deltas })
    }
}
