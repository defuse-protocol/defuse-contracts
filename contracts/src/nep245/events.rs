use std::borrow::Cow;

use derive_more::derive::From;
use near_sdk::{json_types::U128, near, AccountIdRef};

use super::TokenId;

#[must_use = "make sure to `.emit()` this event"]
#[near(event_json(standard = "nep245"))]
// TODO: #[derive(Deserialize)]
#[derive(Debug, From)]
pub enum MtEvent<'a> {
    #[event_version("1.0.0")]
    MtMint(&'a [MtMintEvent<'a>]),
    #[event_version("1.0.0")]
    MtBurn(&'a [MtBurnEvent<'a>]),
    #[event_version("1.0.0")]
    MtTransfer(&'a [MtTransferEvent<'a>]),
}

#[must_use = "make sure to `.emit()` this event"]
#[near(serializers = [json])]
#[derive(Debug)]
pub struct MtMintEvent<'a> {
    pub owner_id: Cow<'a, AccountIdRef>,
    pub token_ids: Cow<'a, [TokenId]>,
    pub amounts: Cow<'a, [U128]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<Cow<'a, str>>,
}

#[must_use = "make sure to `.emit()` this event"]
#[near(serializers = [json])]
#[derive(Debug)]
pub struct MtBurnEvent<'a> {
    pub owner_id: Cow<'a, AccountIdRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<Cow<'a, AccountIdRef>>,
    pub token_ids: Cow<'a, [TokenId]>,
    pub amounts: Cow<'a, [U128]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<Cow<'a, str>>,
}

#[must_use = "make sure to `.emit()` this event"]
#[near(serializers = [json])]
#[derive(Debug)]
pub struct MtTransferEvent<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<Cow<'a, AccountIdRef>>,
    pub old_owner_id: Cow<'a, AccountIdRef>,
    pub new_owner_id: Cow<'a, AccountIdRef>,
    pub token_ids: Cow<'a, [TokenId]>,
    pub amounts: Cow<'a, [U128]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<Cow<'a, str>>,
}

pub trait MtEventEmit<'a>: Into<MtEvent<'a>> {
    #[inline]
    fn emit(self) {
        MtEvent::emit(&self.into())
    }
}
impl<'a, T> MtEventEmit<'a> for T where T: Into<MtEvent<'a>> {}
