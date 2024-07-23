use near_sdk::{ext_contract, near, AccountId, Gas, PromiseOrValue};

#[ext_contract(ext_cross_chain_action)]
pub trait CrossChainTransferReceiver {
    /// Called by oracle when an asset from foreign chain was received.
    /// Returns bool indicating whether to refund received asset back to `sender`
    fn on_cross_chain_transfer(
        &mut self,
        asset: String,
        amount: String,
        msg: String,
        // TODO: meta: tx_hash? proof?
    ) -> PromiseOrValue<bool>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
#[serde(tag = "type", rename = "cross_chain")]
pub struct CrossChainAsset {
    /// Where to expect transfer notification from
    pub oracle: AccountId,
    // TODO: format?
    // eth:1:0x123
    pub asset: String,
    /// In case of NFT, this can be token_id
    pub amount: String,
    // TODO: tx_hash?
    // #[borsh(
    //     serialize_with = "crate::utils::swerde_borsh::serialize_json",
    //     deserialize_with = "crate::utils::serde_borsh::deserialize_json"
    // )]
    // pub extra: HashMap<String, serde_json::Value>,
}

impl CrossChainAsset {
    const GAS_FOR_CROSS_CHAIN_TRANSFER: Gas = Gas::from_gas(0);

    #[must_use]
    #[inline]
    pub const fn gas_for_transfer(&self) -> Gas {
        Self::GAS_FOR_CROSS_CHAIN_TRANSFER
    }

    #[must_use]
    #[inline]
    pub const fn gas_for_refund(&self) -> Gas {
        // should be automatically refunded in cross_chain_resolve_transfer()
        Gas::from_gas(0)
    }

    #[must_use]
    #[inline]
    pub const fn is_zero_amount(&self) -> bool {
        // TODO: `amount` can be used to specify token_id of NFT
        false
    }
}
