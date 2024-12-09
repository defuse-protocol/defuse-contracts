mod nep141;
mod nep171;
mod nep245;

use std::borrow::Cow;

use defuse_core::{tokens::TokenId, DefuseError, Result};
use defuse_nep245::{MtBurnEvent, MtEvent, MtMintEvent};
use near_sdk::{json_types::U128, AccountId, AccountIdRef, Gas};

use super::Contract;

pub const STORAGE_DEPOSIT_GAS: Gas = Gas::from_tgas(10);

impl Contract {
    pub(crate) fn deposit(
        &mut self,
        owner_id: AccountId,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()> {
        let owner = self.accounts.get_or_create(owner_id.clone());

        let mut mint_event = MtMintEvent {
            owner_id: Cow::Borrowed(owner_id.as_ref()),
            token_ids: Default::default(),
            amounts: Default::default(),
            memo: memo.map(Into::into),
        };

        for (token_id, amount) in tokens {
            if amount == 0 {
                return Err(DefuseError::InvalidIntent);
            }

            mint_event.token_ids.to_mut().push(token_id.to_string());
            mint_event.amounts.to_mut().push(U128(amount));

            self.state
                .total_supplies
                .deposit(token_id.clone(), amount)
                .ok_or(DefuseError::BalanceOverflow)?;
            owner
                .token_balances
                .deposit(token_id, amount)
                .ok_or(DefuseError::BalanceOverflow)?;
        }

        MtEvent::MtMint([mint_event].as_slice().into()).emit();

        Ok(())
    }

    pub(crate) fn withdraw(
        &mut self,
        owner_id: &AccountIdRef,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<impl Into<String>>,
    ) -> Result<()> {
        let owner = self
            .accounts
            .get_mut(owner_id)
            .ok_or(DefuseError::AccountNotFound)?;

        let mut burn_event = MtBurnEvent {
            owner_id: Cow::Owned(owner_id.to_owned()),
            authorized_id: None,
            token_ids: Default::default(),
            amounts: Default::default(),
            memo: memo.map(Into::into).map(Into::into),
        };

        for (token_id, amount) in token_amounts {
            if amount == 0 {
                return Err(DefuseError::InvalidIntent);
            }

            burn_event.token_ids.to_mut().push(token_id.to_string());
            burn_event.amounts.to_mut().push(U128(amount));

            owner
                .token_balances
                .withdraw(token_id.clone(), amount)
                .ok_or(DefuseError::BalanceOverflow)?;
            self.state
                .total_supplies
                .withdraw(token_id, amount)
                .ok_or(DefuseError::BalanceOverflow)?;
        }

        // Schedule to emit `mt_burn` events only in the end of tx
        // to avoid confusion when `mt_burn` occurs before relevant
        // `mt_transfer` arrives. This can happen due to postponed
        // delta-matching during intents execution.
        self.postponed_burns.mt_burn(burn_event);

        Ok(())
    }
}
