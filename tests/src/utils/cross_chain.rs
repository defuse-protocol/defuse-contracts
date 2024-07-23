use near_sdk::AccountId;
use serde_json::json;

pub trait OnCrossChainTransferExt {
    async fn on_cross_chain_tranfer(
        &self,
        contract: &AccountId,
        asset: String,
        amount: String,
        msg: String,
    ) -> anyhow::Result<bool>;
}

impl OnCrossChainTransferExt for near_workspaces::Account {
    async fn on_cross_chain_tranfer(
        &self,
        contract: &AccountId,
        asset: String,
        amount: String,
        msg: String,
    ) -> anyhow::Result<bool> {
        self.call(contract, "on_cross_chain_transfer")
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
