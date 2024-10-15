use derive_more::derive::From;
use near_sdk::{json_types::U128, near, serde::Serialize, AccountId};

use super::TokenId;

#[must_use = "make sure to `.emit()` this event"]
#[near(event_json(standard = "nep245"))]
#[derive(Debug, From)]
pub enum MtEvent<'a> {
    #[event_version("0.1.0")]
    MtMint(&'a [MtMintEvent<'a>]),
    #[event_version("0.1.0")]
    MtBurn(&'a [MtBurnEvent<'a>]),
    #[event_version("0.1.0")]
    MtTransfer(&'a [MtTransferEvent<'a>]),
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct MtMintEvent<'a> {
    pub owner_id: &'a AccountId,
    pub token_ids: &'a [TokenId],
    pub amounts: &'a [U128],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>,
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct MtBurnEvent<'a> {
    pub owner_id: &'a AccountId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<&'a AccountId>,
    pub token_ids: &'a [TokenId],
    pub amounts: &'a [U128],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>,
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct MtTransferEvent<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<&'a AccountId>,
    pub old_owner_id: &'a AccountId,
    pub new_owner_id: &'a AccountId,
    pub token_ids: &'a [TokenId],
    pub amounts: &'a [U128],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>,
}

pub trait MtEventEmit<'a>: Into<MtEvent<'a>> {
    #[inline]
    fn emit(self) {
        MtEvent::emit(&self.into())
    }
}
impl<'a, T> MtEventEmit<'a> for T where T: Into<MtEvent<'a>> {}
