pub mod account;
pub mod relayer;
pub mod token_diff;
pub mod tokens;

use derive_more::derive::From;
use near_sdk::{ext_contract, near, serde::Serialize, AccountId, CryptoHash};
use serde_with::serde_as;

use crate::{crypto::SignedPayload, utils::serde::base58::Base58};

use super::{
    fees::FeesManager,
    payload::{DefuseMessage, MultiStandardPayload},
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
    fn execute_intents(&mut self, intents: Vec<SignedPayload<MultiStandardPayload>>) -> Result<()>;

    #[handle_result]
    fn simulate_intents(self, intents: Vec<DefuseMessage<DefuseIntents>>) -> Result<()>;
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct DefuseIntents {
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

    FtWithdraw(FtWithdraw),
    NftWithdraw(NftWithdraw),
    MtWithdraw(MtWithdraw),

    TokenDiff(TokenDiff),
}

#[must_use = "make sure to `.emit()` this event"]
#[serde_as]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct IntentExecutedEvent<'a> {
    pub signer_id: &'a AccountId,

    #[serde_as(as = "Base58")]
    pub hash: &'a CryptoHash,
}

#[cfg(test)]
mod tests {

    use crate::{crypto::Payload, nep413::Nep413Payload};
    use hex_literal::hex;
    use near_sdk::serde_json::{self, json};

    #[test]
    fn test_hash() {
        let p: Nep413Payload = serde_json::from_value(json!({
"message": "{
    \"signer_id\": \"signer.near\",
    \"deadline\": {
        \"timestamp\": 1234567890
    },
    \"intents\": [{
        \"intent\": \"token_diff\",
        \"diff\": {
            \"nep141:ft1.near\": \"-10\",
            \"nep141:ft2.near\": \"20\"
        }
    }]
}",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP8=",
        "recipient": "defuse.near"
        }))
        .unwrap();

        assert_eq!(
            p.hash(),
            hex!("5414a7696afbb648e32e07bf3b1889a0b09c85cde4e00ba32b257d65900a2026")
        );
    }
}
