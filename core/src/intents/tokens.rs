use std::collections::HashMap;

use near_contract_standards::non_fungible_token;
use near_sdk::{json_types::U128, near, AccountId, AccountIdRef, NearToken};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    engine::{Engine, Inspector, State},
    tokens::TokenId,
    DefuseError, Result,
};

use super::ExecutableIntent;

// TODO: decouple from NEP-245, emit out own logs
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Transfer {
    pub receiver_id: AccountId,

    #[serde_as(as = "HashMap<_, DisplayFromStr>")]
    pub tokens: HashMap<TokenId, u128>,

    // TODO: remove due to reduce
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl ExecutableIntent for Transfer {
    fn execute_intent<S, I>(self, sender_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        if sender_id == self.receiver_id || self.tokens.is_empty() {
            return Err(DefuseError::InvalidIntent);
        }
        engine.inspector.on_transfer(sender_id, &self);
        engine
            .state
            .internal_withdraw(sender_id, self.tokens.clone())?;
        engine
            .state
            .internal_deposit(self.receiver_id, self.tokens)?;
        Ok(())
    }
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct FtWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub amount: U128,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    // TODO: msg for whitelisted ft_transfer_call
    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}

impl ExecutableIntent for FtWithdraw {
    #[inline]
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        engine.state.ft_withdraw(owner_id, self)
    }
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct NftWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_id: non_fungible_token::TokenId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    // TODO: msg for whitelisted ft_transfer_call
    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}

impl ExecutableIntent for NftWithdraw {
    #[inline]
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        engine.state.nft_withdraw(owner_id, self)
    }
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct MtWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_ids: Vec<defuse_nep245::TokenId>,
    pub amounts: Vec<U128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    // TODO: msg for whitelisted ft_transfer_call
    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}
impl ExecutableIntent for MtWithdraw {
    #[inline]
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        engine.state.mt_withdraw(owner_id, self)
    }
}

/// Withdraw native NEAR to `receiver_id`.
/// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
/// NOTE: the `wNEAR` will not be refunded in case of fail (e.g. `receiver_id`
/// account does not exist).
#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct NativeWithdraw {
    pub receiver_id: AccountId,
    pub amount: NearToken,
}

impl ExecutableIntent for NativeWithdraw {
    #[inline]
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        engine.state.native_withdraw(owner_id, self)
    }
}
