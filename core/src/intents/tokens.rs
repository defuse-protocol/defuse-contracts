use core::iter;
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
        engine.internal_transfer(sender_id, self)
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
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        let token_amounts = iter::once((TokenId::Nep141(self.token.clone()), self.amount.0)).chain(
            self.storage_deposit.map(|amount| {
                (
                    TokenId::Nep141(engine.state.wnear_id().into_owned()),
                    amount.as_yoctonear(),
                )
            }),
        );

        engine
            .state
            .withdraw(owner_id, token_amounts.clone(), Some("withdraw"))?;

        engine.state.on_ft_withdraw(owner_id, self);
        Ok(())
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
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        let token_amounts = iter::once((
            TokenId::Nep171(self.token.clone(), self.token_id.clone()),
            1,
        ))
        .chain(self.storage_deposit.map(|amount| {
            (
                TokenId::Nep141(engine.state.wnear_id().into_owned()),
                amount.as_yoctonear(),
            )
        }));

        engine
            .state
            .withdraw(owner_id, token_amounts.clone(), Some("withdraw"))?;

        engine.state.on_nft_withdraw(owner_id, self);
        Ok(())
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
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        if self.token_ids.len() != self.amounts.len() || self.token_ids.is_empty() {
            return Err(DefuseError::InvalidIntent);
        }

        let token_amounts = iter::repeat(self.token.clone())
            .zip(self.token_ids.iter().cloned())
            .map(|(token, token_id)| TokenId::Nep245(token, token_id))
            .zip(self.amounts.iter().map(|a| a.0))
            .chain(self.storage_deposit.map(|amount| {
                (
                    TokenId::Nep141(engine.state.wnear_id().into_owned()),
                    amount.as_yoctonear(),
                )
            }));

        engine
            .state
            .withdraw(owner_id, token_amounts.clone(), Some("withdraw"))?;
        engine.state.on_mt_withdraw(owner_id, self);
        Ok(())
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
    fn execute_intent<S, I>(self, owner_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        engine.state.withdraw(
            owner_id,
            [(
                TokenId::Nep141(engine.state.wnear_id().into_owned()),
                self.amount.as_yoctonear(),
            )],
            Some("withdraw"),
        )?;

        engine.state.on_native_withdraw(owner_id, self);
        Ok(())
    }
}
