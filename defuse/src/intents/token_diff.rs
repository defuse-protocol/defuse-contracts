use std::borrow::Cow;

use defuse_contracts::{
    defuse::{
        events::DefuseIntentEmit,
        intents::{token_diff::TokenDiff, AccountEvent},
        tokens::TokenId,
        DefuseError, Result,
    },
    nep245::{self, MtEventEmit, MtTransferEvent},
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

        AccountEvent::new(Cow::Borrowed(signer_id.as_ref()), Cow::Borrowed(&intent)).emit();

        let mut transfer_events = DeltaTransferEventsBuilder::default();
        let fees_collected = self
            .runtime
            .postponed_deposits
            .entry(self.fees.fee_collector.clone())
            .or_default();

        for (token_id, delta) in intent.diff {
            require!(delta != 0, "zero delta");

            signer.token_balances.add_delta(token_id.clone(), delta)?;

            let delta_abs = delta.unsigned_abs();
            let fee = TokenDiff::token_fee(&token_id, delta_abs, self.fees.fee).fee_ceil(delta_abs);
            fees_collected.add(token_id.clone(), fee)?;

            transfer_events.push_delta(&token_id, delta);
            self.runtime.total_supply_deltas.add(
                token_id,
                delta
                    .checked_add_unsigned(fee)
                    .ok_or(DefuseError::BalanceOverflow)?,
            )?;
        }

        transfer_events.emit_for(signer_id, &self.fees.fee_collector);

        Ok(())
    }
}

#[derive(Debug, Default)]
struct DeltaTransferEventsBuilder {
    withdraw_token_ids: Vec<nep245::TokenId>,
    withdraw_amounts: Vec<U128>,
    deposit_token_ids: Vec<nep245::TokenId>,
    deposit_amounts: Vec<U128>,
}

impl DeltaTransferEventsBuilder {
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

    pub fn emit_for(self, signer: &AccountId, collector: &AccountId) {
        [
            (!self.withdraw_amounts.is_empty()).then(|| {
                Self::make_event(
                    signer,
                    collector,
                    &self.withdraw_token_ids,
                    &self.withdraw_amounts,
                )
            }),
            (!self.deposit_amounts.is_empty()).then(|| {
                Self::make_event(
                    collector,
                    signer,
                    &self.deposit_token_ids,
                    &self.deposit_amounts,
                )
            }),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .emit();
    }

    #[inline]
    fn make_event<'a>(
        old_owner_id: &'a AccountId,
        new_owner_id: &'a AccountId,
        token_ids: &'a [nep245::TokenId],
        amounts: &'a [U128],
    ) -> MtTransferEvent<'a> {
        MtTransferEvent {
            authorized_id: None,
            old_owner_id: Cow::Borrowed(old_owner_id.as_ref()),
            new_owner_id: Cow::Borrowed(new_owner_id.as_ref()),
            token_ids: token_ids.into(),
            amounts: amounts.into(),
            memo: Some("token_diff".into()),
        }
    }
}
