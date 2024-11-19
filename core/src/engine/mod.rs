mod runtime;
mod state;

use defuse_nep245::MtEvent;
use derive_more::derive::From;
use impl_tools::autoimpl;
use near_sdk::{serde::Serialize, AccountIdRef};

use crate::{
    events::DefuseEvent,
    intents::tokens::{FtWithdraw, MtBatchTransfer, MtWithdraw, NftWithdraw},
    Deadline, Result,
};

pub use self::{runtime::*, state::*};

// #[derive(Debug)]
// pub struct Engine<H: Handler> {
//     pub(crate) handler: H,
//     pub(crate) runtime: Runtime,
// }

// impl<H> Engine<H>
// where
//     H: Handler,
// {
//     #[inline]
//     pub fn new(handler: H) -> Self {
//         Self {
//             handler,
//             runtime: Default::default(),
//         }
//     }

//     #[inline]
//     pub fn into_runtime(self) -> Runtime {
//         self.runtime
//     }

//     #[inline]
//     pub fn finalize(self) -> Result<()> {
//         self.runtime.finalize()
//     }
// }

// #[autoimpl(for<T: trait + ?Sized> &mut T, Box<T>)]
// pub trait Handler: State {
//     fn on_intent_deadline(&mut self, deadline: Deadline);

//     fn emit<'a>(&mut self, event: impl Into<Event<'a>>);

//     // fn on_mt_batch_transfer(
//     //     &mut self,
//     //     sender_id: &AccountIdRef,
//     //     transfer: MtBatchTransfer,
//     // ) -> Result<()>;

//     // fn on_ft_withdraw(&mut self, sender_id: &AccountIdRef, withdraw: FtWithdraw) -> Result<()>;
//     // fn on_nft_withdraw(&mut self, sender_id: &AccountIdRef, withdraw: NftWithdraw) -> Result<()>;
//     // fn on_mt_withdraw(&mut self, sender_id: &AccountIdRef, withdraw: MtWithdraw) -> Result<()>;
// }

// #[derive(Debug, Clone, From, Serialize)]
// #[serde(crate = "::near_sdk::serde", untagged)]
// pub enum Event<'a> {
//     Defuse(DefuseEvent<'a>),
//     Mt(MtEvent<'a>),
// }

// impl<'a> Event<'a> {
//     pub fn emit(&self) {
//         match self {
//             Self::Defuse(event) => event.emit(),
//             Self::Mt(event) => event.emit(),
//         }
//     }
// }
