pub mod account;
pub mod token_diff;
pub mod tokens;

use defuse_crypto::{PublicKey, SignedPayload};
use defuse_serde_utils::base58::Base58;
use derive_more::derive::From;
use near_sdk::{near, AccountIdRef, CryptoHash};
use serde_with::serde_as;

use crate::{
    accounts::AccountEvent,
    engine::State,
    events::DefuseEvent,
    payload::{DefusePayload, ExtractDefusePayload},
    DefuseError, Result,
};

use self::{
    account::{AddPublicKey, InvalidateNonces, RemovePublicKey},
    token_diff::TokenDiff,
    tokens::{FtWithdraw, MtBatchTransfer, MtWithdraw, NftWithdraw},
};

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct DefuseIntents {
    /// Sequence of intents to execute in given order. Empty list is also
    /// a valid sequence, i.e. it doesn't do anything, but still invalidates
    /// the `nonce` for the signer
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intents: Vec<Intent>,
}

#[near(serializers = [borsh, json])]
#[serde(tag = "intent", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum Intent {
    AddPublicKey(AddPublicKey),
    RemovePublicKey(RemovePublicKey),
    InvalidateNonces(InvalidateNonces),

    MtBatchTransfer(MtBatchTransfer),

    FtWithdraw(FtWithdraw),
    NftWithdraw(NftWithdraw),
    MtWithdraw(MtWithdraw),

    TokenDiff(TokenDiff),
}

pub trait ExecutableIntent {
    fn execute_intent<S>(self, signer_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State;
}

// impl<H> Engine<H>
// where
//     H: Handler,
// {
//     pub fn execute_signed_intents<I, P>(
//         &mut self,
//         payloads: impl IntoIterator<Item = P>,
//     ) -> Result<()>
//     where
//         P: SignedPayload<PublicKey = PublicKey> + ExtractDefusePayload<I>,
//         I: ExecutableIntent,
//         DefuseError: From<<P as ExtractDefusePayload<I>>::Error>,
//     {
//         let events = payloads
//             .into_iter()
//             .map(|p| self.execute_signed_intent(p))
//             .collect::<Result<Vec<_>>>()?;

//         self.handler
//             .emit(DefuseEvent::IntentsExecuted(events.into()));
//         Ok(())
//     }

//     fn execute_signed_intent<I, P>(
//         &mut self,
//         payload: P,
//     ) -> Result<AccountEvent<'static, IntentExecutedEvent>>
//     where
//         P: SignedPayload<PublicKey = PublicKey> + ExtractDefusePayload<I>,
//         I: ExecutableIntent,
//         DefuseError: From<<P as ExtractDefusePayload<I>>::Error>,
//     {
//         // calculate intent hash
//         let hash = payload.hash();

//         // verify signature
//         let public_key = payload.verify().ok_or(DefuseError::InvalidSignature)?;

//         // extract Defuse payload
//         let DefusePayload::<I> {
//             signer_id,
//             verifying_contract,
//             deadline,
//             nonce,
//             message: intent,
//         } = payload.extract_defuse_payload()?;

//         // check recipient
//         if verifying_contract != self.handler.verifying_contract().as_ref() {
//             return Err(DefuseError::WrongRecipient);
//         }

//         // make sure message is still valid
//         if deadline.has_expired() {
//             return Err(DefuseError::DeadlineExpired);
//         }

//         // make sure the account has this public key
//         if !self.handler.has_public_key(&signer_id, &public_key) {
//             return Err(DefuseError::PublicKeyNotExist);
//         }

//         // commit nonce
//         if !self.handler.commit_nonce(signer_id.clone(), nonce) {
//             return Err(DefuseError::NonceUsed);
//         };

//         // execute intent
//         intent.execute_intent(&signer_id, self)?;

//         Ok(AccountEvent::new(signer_id, IntentExecutedEvent { hash }))
//     }
// }

impl ExecutableIntent for DefuseIntents {
    fn execute_intent<S>(self, signer_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        for intent in self.intents {
            intent.execute_intent(signer_id, state)?;
        }
        Ok(())
    }
}

impl ExecutableIntent for Intent {
    fn execute_intent<S>(self, signer_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        match self {
            Self::AddPublicKey(intent) => intent.execute_intent(signer_id, state),
            Self::RemovePublicKey(intent) => intent.execute_intent(signer_id, state),
            Self::InvalidateNonces(intent) => intent.execute_intent(signer_id, state),
            Self::MtBatchTransfer(intent) => intent.execute_intent(signer_id, state),
            Self::FtWithdraw(intent) => intent.execute_intent(signer_id, state),
            Self::NftWithdraw(intent) => intent.execute_intent(signer_id, state),
            Self::MtWithdraw(intent) => intent.execute_intent(signer_id, state),
            Self::TokenDiff(intent) => intent.execute_intent(signer_id, state),
        }
    }
}

#[must_use = "make sure to `.emit()` this event"]
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [json])]
#[derive(Debug, Clone)]
pub struct IntentExecutedEvent {
    #[serde_as(as = "Base58")]
    pub hash: CryptoHash,
}
