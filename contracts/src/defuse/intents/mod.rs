pub mod account;
pub mod relayer;
pub mod token_diff;
pub mod tokens;

use derive_more::derive::From;
use near_sdk::{ext_contract, near};

use crate::utils::Deadline;

use super::{
    fees::FeesManager,
    payload::{DefusePayload, SignedDefusePayload},
    Result,
};

use self::{
    account::{AddPublicKey, InvalidateNonces, RemovePublicKey},
    token_diff::TokenDiff,
    tokens::{FtWithdraw, MtBatchTransfer, MtBatchTransferCall, MtWithdraw, NftWithdraw},
};

#[ext_contract(ext_intents_executor)]
pub trait IntentsExecutor: FeesManager {
    #[handle_result]
    fn execute_intents(
        &mut self,
        #[serializer(borsh)] intents: Vec<SignedDefusePayload<DefuseIntents>>,
    ) -> Result<()>;
    #[handle_result]
    fn execute_intents_json(
        &mut self,
        intents: Vec<SignedDefusePayload<DefuseIntents>>,
    ) -> Result<()>;

    #[handle_result]
    fn simulate_intents(
        self,
        #[serializer(borsh)] intents: Vec<DefusePayload<DefuseIntents>>,
    ) -> Result<()>;
    #[handle_result]
    fn simulate_intents_json(self, intents: Vec<DefusePayload<DefuseIntents>>) -> Result<()>;
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct DefuseIntents {
    pub deadline: Deadline,

    /// Sequence of intents to execute in given order. Empty list is also
    /// a valid sequence, i.e. it doesn't do anything, but still invalidates
    /// the [`nonce`](crate::nep413::Nep413Payload::nonce) for the signer
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intents: Vec<Intent>,
}

#[near(serializers = [borsh, json])]
#[serde(tag = "intent", rename_all = "snake_case")]
#[derive(Debug, Clone, From)]
pub enum Intent {
    AddPublicKey(AddPublicKey),
    RemovePublicKey(RemovePublicKey),
    InvalidateNonces(InvalidateNonces),

    MtBatchTransfer(MtBatchTransfer),
    MtBatchTransferCall(MtBatchTransferCall),

    TokenDiff(TokenDiff),

    FtWithdraw(FtWithdraw),
    NftWithdraw(NftWithdraw),
    MtWithdraw(MtWithdraw),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::crypto::Payload;
    use hex_literal::hex;
    use near_sdk::serde_json::{self, json};

    #[test]
    fn test_hash() {
        let p: DefusePayload<DefuseIntents> = serde_json::from_value(json!({
          "message": {
            "signer_id": "signer.near",
            "deadline": {
              "timestamp": 1234567890,
            },
            "intents": [
              {
                "intent": "token_diff",
                "diff": {
                    "nep141:ft1.near": "-10",
                    "nep141:ft2.near": "20"
                }
              }
            ]
          },
          "nonce": "1",
          "recipient": "defuse.near"
        }))
        .unwrap();

        assert_eq!(
            p.hash(),
            hex!("f33733baae0120c85180683b8cae4ec8ca6d2082886523cb59a2d69ff4163ebe")
        );
    }
}
