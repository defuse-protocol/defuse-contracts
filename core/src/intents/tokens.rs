use core::iter;
use std::borrow::Cow;

use defuse_nep245::{MtBurnEvent, MtEvent};
use near_contract_standards::non_fungible_token;
use near_sdk::{json_types::U128, near, AccountId, AccountIdRef, NearToken};

use crate::{engine::State, tokens::TokenId, DefuseError, Result};

use super::ExecutableIntent;

// impl<H> Engine<H>
// where
//     H: Handler,
// {
//     #[inline]
//     pub fn wnear_token_id(&self) -> TokenId {
//         TokenId::Nep141(self.handler.wnear_id().clone().into_owned())
//     }

//     // pub fn withdraw(
//     //     &mut self,
//     //     owner_id: &AccountIdRef,
//     //     token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
//     //     memo: Option<&str>,
//     // ) -> Result<()> {
//     //     let mut token_ids = Vec::new();
//     //     let mut amounts = Vec::new();

//     //     for (token_id, amount) in token_amounts {
//     //         if amount == 0 {
//     //             return Err(DefuseError::ZeroAmount);
//     //         }

//     //         token_ids.push(token_id.to_string());
//     //         amounts.push(U128(amount));

//     //         self.handler
//     //             .withdraw(owner_id.to_owned(), token_id, amount)
//     //             .ok_or(DefuseError::BalanceOverflow)?;
//     //     }

//     //     if token_ids.is_empty() {
//     //         return Err(DefuseError::ZeroAmount);
//     //     }

//     //     self.handler.emit(MtEvent::MtBurn(
//     //         [MtBurnEvent {
//     //             owner_id: Cow::Borrowed(owner_id),
//     //             authorized_id: None,
//     //             token_ids: token_ids.into(),
//     //             amounts: amounts.into(),
//     //             memo: memo.map(Into::into),
//     //         }]
//     //         .as_slice()
//     //         .into(),
//     //     ));

//     //     Ok(())
//     // }
// }

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct MtBatchTransfer {
    pub receiver_id: AccountId,
    pub token_ids: Vec<defuse_nep245::TokenId>,
    pub amounts: Vec<U128>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// `msg` to pass in `mt_on_transfer`
    pub msg: Option<String>,
}

impl ExecutableIntent for MtBatchTransfer {
    fn execute_intent<S>(self, sender_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        state.mt_transfer(sender_id.to_owned(), self)
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

    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}

impl ExecutableIntent for FtWithdraw {
    fn execute_intent<S>(self, owner_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        state.ft_withdraw(owner_id.to_owned(), self)
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

    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}

impl ExecutableIntent for NftWithdraw {
    fn execute_intent<S>(self, owner_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        state.nft_withdraw(owner_id.to_owned(), self)
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

    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}
impl ExecutableIntent for MtWithdraw {
    fn execute_intent<S>(self, owner_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        state.mt_withdraw(owner_id.to_owned(), self)
    }
}
