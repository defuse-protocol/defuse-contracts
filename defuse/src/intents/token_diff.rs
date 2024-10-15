use defuse_contracts::{
    defuse::{
        events::DefuseIntentEmit,
        intents::token_diff::{TokenDiff, TokenDiffEvent},
        tokens::TokenId,
        DefuseError, Result,
    },
    nep245::{self, MtEventEmit, MtTransferEvent},
    utils::cache::CURRENT_ACCOUNT_ID,
};
use near_sdk::{json_types::U128, require, AccountId};

use crate::accounts::Account;

use super::{IntentExecutor, State};

impl IntentExecutor<TokenDiff> for State {
    fn execute_intent(
        &mut self,
        signer_id: &AccountId,
        signer: &mut Account,
        intent: TokenDiff,
    ) -> Result<()> {
        require!(!intent.diff.is_empty(), "empty token_diff");

        TokenDiffEvent {
            signer_id,
            token_diff: &intent,
        }
        .emit();
        let mut events = TransferEventsBuilder::default();

        let fees_collected = self
            .runtime
            .postponed_deposits
            .entry(self.fee_collector.clone())
            .or_default();

        for (token_id, delta) in intent.diff {
            require!(delta != 0, "zero delta");

            events.push_delta(&token_id, delta);
            signer.token_balances.add_delta(token_id.clone(), delta)?;

            let fee = self.fee.fee_ceil(delta.unsigned_abs());
            fees_collected.add(token_id.clone(), fee)?;

            self.runtime.total_supply_deltas.add(
                token_id.clone(),
                delta
                    .checked_add_unsigned(fee)
                    .ok_or(DefuseError::IntegerOverflow)?,
            )?;
        }

        events.emit_for(signer_id);

        Ok(())
    }
}

#[derive(Debug, Default)]
struct TransferEventsBuilder {
    withdraw_token_ids: Vec<nep245::TokenId>,
    withdraw_amounts: Vec<U128>,
    deposit_token_ids: Vec<nep245::TokenId>,
    deposit_amounts: Vec<U128>,
}

impl TransferEventsBuilder {
    #[inline]
    pub fn push_delta(&mut self, token_id: &TokenId, delta: i128) {
        let (token_ids, amounts) = if delta.is_negative() {
            (&mut self.withdraw_token_ids, &mut self.withdraw_amounts)
        } else {
            (&mut self.deposit_token_ids, &mut self.deposit_amounts)
        };
        token_ids.push(token_id.to_string());
        amounts.push(U128(delta.unsigned_abs()));
    }

    pub fn emit_for(self, signer: &AccountId) {
        [
            (!self.withdraw_amounts.is_empty()).then(|| {
                Self::make_event(
                    signer,
                    false,
                    &self.withdraw_token_ids,
                    &self.withdraw_amounts,
                )
            }),
            (!self.deposit_amounts.is_empty()).then(|| {
                Self::make_event(signer, true, &self.deposit_token_ids, &self.deposit_amounts)
            }),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .emit();
    }

    #[inline]
    fn make_event<'a>(
        signer: &'a AccountId,
        deposit: bool,
        token_ids: &'a [nep245::TokenId],
        amounts: &'a [U128],
    ) -> MtTransferEvent<'a> {
        MtTransferEvent {
            authorized_id: None,
            old_owner_id: if deposit { &CURRENT_ACCOUNT_ID } else { signer },
            new_owner_id: if deposit { signer } else { &CURRENT_ACCOUNT_ID },
            token_ids,
            amounts,
            memo: Some("token_diff"),
        }
    }
}
