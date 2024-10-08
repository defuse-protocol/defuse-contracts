pub mod token_diff;
pub mod tokens;

use derive_more::derive::From;
use near_sdk::{ext_contract, near};
use serde_with::{serde_as, DisplayFromStr};

use crate::{crypto::PublicKey, nep413::Nonce};

use super::{fees::FeesManager, message::SignedDefuseMessage, Result};

use self::{
    token_diff::TokenDiff,
    tokens::{FtWithdraw, MtBatchTransfer, MtBatchTransferCall, MtWithdraw, NftWithdraw},
};

#[ext_contract(ext_signed_executor)]
pub trait SignedIntentExecutor: FeesManager {
    #[handle_result]
    fn execute_signed_intents(
        &mut self,
        signed: Vec<SignedDefuseMessage<DefuseIntents>>,
    ) -> Result<()>;
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, Default)]
pub struct DefuseIntents {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intents: Vec<Intent>,
}

#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[near(serializers = [borsh, json])]
#[serde(tag = "intent", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum Intent {
    #[from(skip)]
    AddPublicKey {
        public_key: PublicKey,
    },
    #[from(skip)]
    RemovePublicKey {
        public_key: PublicKey,
    },
    #[from(skip)]
    InvalidateNonces {
        #[serde_as(as = "Vec<DisplayFromStr>")]
        nonces: Vec<Nonce>,
    },

    MtBatchTransfer(MtBatchTransfer),
    MtBatchTransferCall(MtBatchTransferCall),

    TokensDiff(TokenDiff),

    FtWithdraw(FtWithdraw),
    NftWithdraw(NftWithdraw),
    MtWithdraw(MtWithdraw),
}
