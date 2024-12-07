use std::collections::HashMap;

use defuse_crypto::{PublicKey, SignedPayload};
use near_sdk::{AccountId, AccountIdRef};

use crate::{
    intents::ExecutableIntent,
    payload::{DefusePayload, ExtractDefusePayload},
    tokens::{TokenAmounts, TokenId},
    DefuseError, Result,
};

use super::{DeltaMatcher, Inspector, State, Transfers};

pub struct Engine<S, I> {
    pub state: S,
    pub inspector: I,
    pub deltas: DeltaMatcher,
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

    pub fn execute_signed_intents<P, T>(
        &mut self,
        signed: impl IntoIterator<Item = P>,
    ) -> Result<()>
    where
        P: SignedPayload<PublicKey = PublicKey> + ExtractDefusePayload<T>,
        T: ExecutableIntent,
        DefuseError: From<<P as ExtractDefusePayload<T>>::Error>,
    {
        for signed in signed {
            self.execute_signed_intent(signed)?;
        }
        Ok(())
    }

    pub fn execute_signed_intent<P, T>(&mut self, signed: P) -> Result<()>
    where
        P: SignedPayload<PublicKey = PublicKey> + ExtractDefusePayload<T>,
        T: ExecutableIntent,
        DefuseError: From<<P as ExtractDefusePayload<T>>::Error>,
    {
        // verify signed payload and get public key
        let public_key = signed.verify().ok_or(DefuseError::InvalidSignature)?;

        // calculate intent hash
        let hash = signed.hash();

        // extract NEP-413 payload
        let DefusePayload::<T> {
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

        self.execute_intent(&signer_id, intents)?;
        self.inspector.on_intent_executed(&signer_id, hash);

        Ok(())
    }

    #[inline]
    pub fn execute_intent<T>(&mut self, signer_id: &AccountIdRef, intent: T) -> Result<()>
    where
        T: ExecutableIntent,
    {
        intent.execute_intent(signer_id, self)
    }

    // pub(crate) fn withdraw(
    //     &mut self,
    //     owner_id: AccountId,
    //     token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    // ) -> Result<()> {
    //     let owner_withdrawals = self.postponed_withdrawals.entry(owner_id).or_default();
    //     for (token_id, amount) in token_amounts {
    //         owner_withdrawals.withdraw(token_id, amount).ok_or(err)?;
    //     }
    //     Ok(())
    // }

    // pub(crate) fn internal_transfer(
    //     &mut self,
    //     sender_id: AccountId,
    //     receiver_id: AccountId,
    //     token_amounts: impl IntoIterator<Item = (TokenId, u128)> + Clone,
    // ) -> Result<()> {
    //     // TODO: own log?
    //     self.postponed_transfers
    //         .withdraw(sender_id, token_amounts.clone())?;
    //     self.postponed_transfers.deposit(receiver_id, token_amounts)
    // }

    pub(crate) fn internal_add_delta(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        delta: i128,
    ) -> Result<()> {
        let token_amounts = [(token_id.clone(), delta.unsigned_abs())];
        if delta.is_negative() {
            self.state.internal_withdraw(&owner_id, token_amounts)?;
        } else {
            // TODO: postpone the deposit?
            // because other users' intents should not depend on the
            // order of intents in execute_intents()?
            // on the other hand: user will not be able to pay FE fee
            // in token_out
            self.state
                .internal_deposit(owner_id.clone(), token_amounts)?;
        }
        self.deltas
            .add_delta(owner_id, token_id, delta)
            .ok_or(DefuseError::BalanceOverflow)?;
        Ok(())
    }

    pub fn finalize(self) -> Result<Transfers> {
        self.deltas.finalize()
    }
}
