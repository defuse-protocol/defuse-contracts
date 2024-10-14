use derive_more::derive::From;
use near_sdk::near;

use super::{
    accounts::{PublicKeyAddedEvent, PublicKeyRemovedEvent},
    tokens::{
        nep141::{FtDepositEvent, FtWithdrawEvent},
        nep171::{NftDepositEvent, NftWithdrawEvent},
    },
};

#[must_use = "make sure to `.emit()` this event"]
#[near(event_json(standard = "dip3"))]
#[derive(Debug, From)]
pub enum DefuseEvent<'a> {
    #[event_version("0.1.0")]
    PublicKeyAdded(PublicKeyAddedEvent<'a>),
    #[event_version("0.1.0")]
    PublicKeyRemoved(PublicKeyRemovedEvent<'a>),
    #[event_version("0.1.0")]
    FtDeposit(FtDepositEvent<'a>),
    #[event_version("0.1.0")]
    FtWithdraw(FtWithdrawEvent<'a>),
    #[event_version("0.1.0")]
    NftDeposit(NftDepositEvent<'a>),
    #[event_version("0.1.0")]
    NftWithdraw(NftWithdrawEvent<'a>),
}

pub trait DefuseIntentEmit<'a>: Into<DefuseEvent<'a>> {
    #[inline]
    fn emit(self) {
        DefuseEvent::emit(&self.into())
    }
}
impl<'a, T> DefuseIntentEmit<'a> for T where T: Into<DefuseEvent<'a>> {}
