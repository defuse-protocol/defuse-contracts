use near_sdk::{ext_contract, json_types::U128, AccountId, Promise};

#[ext_contract(ext_poa_factory)]
pub trait POAFactory {
    // TODO: payable
    fn deploy_token(&mut self, token: String) -> Promise;
    /// TOOD: 1yN
    fn ft_mint(
        &mut self,
        token: String,
        owner_id: AccountId,
        amount: U128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> Promise;
}
