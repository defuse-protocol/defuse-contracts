use std::{borrow::Cow, collections::HashMap, iter};

use defuse_map_utils::cleanup::DefaultMap;
use defuse_nep245::{MtEvent, MtTransferEvent};
use near_sdk::{json_types::U128, AccountId};

use crate::tokens::TokenId;

// TODO: docs
/// Accumulates transfers between
#[derive(Debug, Default)]
pub struct Transfers(
    /// sender_id -> receiver_id -> token_id -> amount
    HashMap<AccountId, HashMap<AccountId, HashMap<TokenId, u128>>>,
);

impl Transfers {
    #[must_use]
    pub fn transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        amount: u128,
    ) -> Option<u128> {
        let mut sender = self.0.entry_or_default(sender_id);
        let mut receiver = sender.entry_or_default(receiver_id);
        let mut transfer = receiver.entry_or_default(token_id);
        *transfer = transfer.checked_add(amount)?;
        Some(*transfer)
    }

    pub fn as_mt_event(&self) -> Option<MtEvent<'_>> {
        if self.0.is_empty() {
            return None;
        }
        Some(MtEvent::MtTransfer(
            self.0
                .iter()
                .flat_map(|(sender_id, transfers)| iter::repeat(sender_id).zip(transfers))
                .map(|(sender_id, (receiver_id, transfers))| {
                    let (token_ids, amounts) = transfers
                        .iter()
                        .map(|(token_id, amount)| (token_id.to_string(), U128(*amount)))
                        .unzip();
                    MtTransferEvent {
                        authorized_id: None,
                        old_owner_id: Cow::Borrowed(sender_id),
                        new_owner_id: Cow::Borrowed(receiver_id),
                        token_ids: Cow::Owned(token_ids),
                        amounts: Cow::Owned(amounts),
                        memo: None,
                    }
                })
                .collect::<Vec<_>>()
                .into(),
        ))
    }
}
