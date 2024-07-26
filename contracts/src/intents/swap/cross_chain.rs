use near_sdk::{ext_contract, near, AccountId, Gas, PromiseOrValue};

#[ext_contract(ext_cross_chain_action)]
pub trait CrossChainReceiver {
    /// Called by oracle when an asset from foreign chain was received.
    /// Returns bool indicating whether to refund received asset back to `sender`
    fn cross_chain_on_transfer(
        &mut self,
        asset: String,
        amount: String,
        msg: String,
    ) -> PromiseOrValue<bool>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
#[serde(tag = "type", rename = "cross_chain")]
pub struct CrossChainAsset {
    /// Where to expect transfer notification from
    pub oracle: AccountId,
    /// Universal cross-chain asset identifier in the following format:
    /// `<CHAIN_TYPE>:<CHAIN_ID>:<ASSET_ADDRESS>`.
    pub asset: String,
    /// In case of NFT, this can be token_id
    pub amount: String,
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
