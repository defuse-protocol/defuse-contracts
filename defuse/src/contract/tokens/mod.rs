mod nep141;
mod nep171;
mod nep245;

use std::borrow::Cow;

use defuse_core::{
    tokens::{TokenAmounts, TokenId},
    DefuseError, Result,
};
use defuse_nep245::{MtBurnEvent, MtEvent, MtMintEvent};
use near_sdk::{json_types::U128, store::IterableMap, AccountId, Gas};

use super::Contract;

pub const STORAGE_DEPOSIT_GAS: Gas = Gas::from_tgas(10);

pub type TokenBalances = TokenAmounts<IterableMap<TokenId, u128>>;

impl Contract {
    // pub(crate) fn internal_deposit(
    //     &mut self,
    //     owner_id: AccountId,
    //     token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    //     memo: Option<&str>,
    // ) -> Result<()> {
    //     let owner = self.accounts.get_or_create(owner_id.clone());

    //     let mut event = MtMintEvent {
    //         owner_id: Cow::Owned(owner_id),
    //         token_ids: Default::default(),
    //         amounts: Default::default(),
    //         memo: memo.map(Into::into),
    //     };

    //     for (token_id, amount) in token_amounts {
    //         if amount == 0 {
    //             return Err(DefuseError::ZeroAmount);
    //         }

    //         event.token_ids.to_mut().push(token_id.to_string());
    //         event.amounts.to_mut().push(U128(amount));

    //         self.state
    //             .total_supplies
    //             .deposit(token_id.clone(), amount)
    //             .ok_or(DefuseError::BalanceOverflow)?;
    //         owner
    //             .token_balances
    //             .deposit(token_id, amount)
    //             .ok_or(DefuseError::BalanceOverflow)?;
    //     }

    //     MtEvent::MtMint([event].as_slice().into()).emit();

    //     Ok(())
    // }

    // pub(crate) fn internal_withdraw(
    //     &mut self,
    //     owner_id: &AccountId,
    //     token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    //     memo: Option<&str>,
    // ) -> Result<()> {
    //     let owner = self
    //         .accounts
    //         .get_mut(&owner_id)
    //         .ok_or(DefuseError::AccountNotFound)?;

    //     let mut event = MtBurnEvent {
    //         owner_id: Cow::Borrowed(owner_id.as_ref()),
    //         authorized_id: None,
    //         token_ids: Default::default(),
    //         amounts: Default::default(),
    //         memo: memo.map(Into::into),
    //     };

    //     for (token_id, amount) in token_amounts {
    //         if amount == 0 {
    //             return Err(DefuseError::ZeroAmount);
    //         }

    //         event.token_ids.to_mut().push(token_id.to_string());
    //         event.amounts.to_mut().push(U128(amount));

    //         owner
    //             .token_balances
    //             .withdraw(token_id.clone(), amount)
    //             .ok_or(DefuseError::BalanceOverflow)?;
    //         self.state
    //             .total_supplies
    //             .withdraw(token_id, amount)
    //             .ok_or(DefuseError::BalanceOverflow)?;
    //     }

    //     MtEvent::MtBurn([event].as_slice().into()).emit();

    //     Ok(())
    // }
}
