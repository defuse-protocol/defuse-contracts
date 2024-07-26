use near_sdk::AccountId;
use serde_json::json;

pub trait CrossChainReceiverExt {
    async fn cross_chain_on_transfer(
        &self,
        contract: &AccountId,
        asset: String,
        amount: String,
        msg: String,
    ) -> anyhow::Result<bool>;
}

impl CrossChainReceiverExt for near_workspaces::Account {
    async fn cross_chain_on_transfer(
        &self,
        contract: &AccountId,
        asset: String,
        amount: String,
        msg: String,
    ) -> anyhow::Result<bool> {
        self.call(contract, "cross_chain_on_transfer")
            .args_json(json!({
                "asset": asset,
                "amount": amount,
                "msg": msg,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?
            .json()
            .map_err(Into::into)
    }
}
