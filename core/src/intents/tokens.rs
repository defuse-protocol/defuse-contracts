use core::iter;

use near_contract_standards::non_fungible_token;
use near_sdk::{json_types::U128, near, AccountId, AccountIdRef, NearToken};

use crate::{
    engine::{Engine, Inspector, State},
    tokens::TokenId,
    DefuseError, Result,
};

use super::ExecutableIntent;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct MtBatchTransfer {
    pub receiver_id: AccountId,
    pub token_ids: Vec<TokenId>,
    pub amounts: Vec<U128>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    // TODO: feature_flag
    /// `msg` to pass in `mt_on_transfer`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
}

impl ExecutableIntent for MtBatchTransfer {
    fn execute_intent<S, I>(self, sender_id: &AccountIdRef, engine: &mut Engine<S, I>) -> Result<()>
    where
        S: State,
        I: Inspector,
    {
        if sender_id == self.receiver_id
            || self.token_ids.is_empty()
            || self.token_ids.len() != self.amounts.len()
        {
            return Err(DefuseError::ZeroAmount);
        }

        let token_amounts = self
            .token_ids
            .iter()
            .cloned()
            .zip(self.amounts.iter().map(|a| a.0));

        engine
            .state
            .internal_withdraw(sender_id, token_amounts.clone())?;
        engine
            .state
            .internal_deposit(self.receiver_id.clone(), token_amounts.clone())?;

        engine.state.on_mt_transfer(sender_id, self);
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
            return Err(DefuseError::ZeroAmount);
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
