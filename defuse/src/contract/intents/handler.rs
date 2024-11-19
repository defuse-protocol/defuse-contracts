use defuse_core::{
    intents::{
        tokens::{FtWithdraw, MtBatchTransfer, MtWithdraw, NftWithdraw},
        DefuseIntents,
    },
    payload::DefusePayload,
    Deadline, Result,
};

use near_sdk::AccountIdRef;

use crate::contract::Contract;

// impl Handler for Contract {
//     fn on_intent_deadline(&mut self, _deadline: Deadline) {}

//     fn emit<'a>(&mut self, event: impl Into<Event<'a>>) {
//         event.into().emit()
//     }

//     fn on_mt_batch_transfer(
//         &mut self,
//         sender_id: &AccountIdRef,
//         MtBatchTransfer {
//             receiver_id,
//             token_ids,
//             amounts,
//             msg,
//             ..
//         }: MtBatchTransfer,
//     ) -> Result<()> {
//         if let Some(msg) = msg {
//             // detach promise
//             let _ = Self::internal_mt_batch_transfer_call(
//                 sender_id.to_owned(),
//                 receiver_id,
//                 token_ids.iter().map(ToString::to_string).collect(),
//                 amounts,
//                 msg,
//             );
//         }

//         Ok(())
//     }

//     fn on_ft_withdraw(&mut self, sender_id: &AccountIdRef, withdraw: FtWithdraw) -> Result<()> {
//         let _ = self.internal_ft_withdraw(sender_id.to_owned(), withdraw);
//         Ok(())
//     }

//     fn on_nft_withdraw(&mut self, sender_id: &AccountIdRef, withdraw: NftWithdraw) -> Result<()> {
//         todo!()
//     }

//     fn on_mt_withdraw(&mut self, sender_id: &AccountIdRef, withdraw: MtWithdraw) -> Result<()> {
//         todo!()
//     }
// }
