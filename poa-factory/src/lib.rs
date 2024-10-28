use defuse_contracts::{
    poa::{factory::POAFactory, token::ext_poa_fungible_token},
    utils::{cache::CURRENT_ACCOUNT_ID, UnwrapOrPanicError},
};
use near_account_id::ParseAccountError;
use near_contract_standards::fungible_token::{core::ext_ft_core, Balance};
use near_sdk::{
    borsh::BorshSerialize,
    env,
    json_types::U128,
    near, require,
    serde_json::{self, json},
    store::IterableSet,
    AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise,
};

const POA_TOKEN_WASM: &'static [u8] = include_bytes!(std::env!(
    "POA_TOKEN_WASM",
    "Set ${POA_TOKEN_WASM} to be the path of the PoA token binary",
));

const POA_TOKEN_INIT_BALANCE: NearToken = NearToken::from_near(3);
const POA_TOKEN_NEW_GAS: Gas = Gas::from_tgas(10);
const POA_TOKEN_FT_MINT_GAS: Gas = Gas::from_tgas(10);

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct POAFactoryImpl {
    tokens: IterableSet<String>,
    bridge_token_storage_deposit_required: NearToken,
}

#[near]
impl POAFactoryImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            tokens: IterableSet::new(Prefix::Tokens),
            bridge_token_storage_deposit_required: env::storage_byte_cost().saturating_mul(
                near_contract_standards::fungible_token::FungibleToken::new(b"t")
                    .account_storage_usage as Balance,
            ),
        }
    }
}

#[near]
impl POAFactory for POAFactoryImpl {
    #[payable]
    fn deploy_token(&mut self, token: String) -> Promise {
        let token_id = Self::token_id(token.clone()).unwrap_or_panic_display();

        let initial_storage = env::storage_usage() as u128;
        require!(self.tokens.insert(token), "token exists");
        let current_storage = env::storage_usage() as u128;
        require!(
            env::attached_deposit()
                >= POA_TOKEN_INIT_BALANCE.saturating_add(
                    env::storage_byte_cost()
                        .saturating_mul(current_storage.saturating_sub(initial_storage))
                ),
            "not enough attached deposit to deploy PoA token"
        );

        Promise::new(token_id)
            .create_account()
            .transfer(POA_TOKEN_INIT_BALANCE)
            .deploy_contract(POA_TOKEN_WASM.to_vec())
            .function_call(
                "new".to_string(),
                // TODO: metadata?
                serde_json::to_vec(&json!({})).unwrap_or_panic_display(),
                NearToken::from_yoctonear(0),
                POA_TOKEN_NEW_GAS,
            )
    }

    // TODO: ACL
    #[payable]
    fn ft_mint(
        &mut self,
        token: String,
        owner_id: AccountId,
        amount: U128,
        msg: Option<String>,
        memo: Option<String>,
    ) -> Promise {
        require!(
            env::attached_deposit() >= self.bridge_token_storage_deposit_required,
            "not enough attached deposit"
        );

        require!(self.tokens.contains(&token), "token does not exist");

        let token_id = Self::token_id(token).unwrap_or_panic_display();

        if let Some(msg) = msg {
            ext_poa_fungible_token::ext(token_id.clone())
                .with_attached_deposit(env::attached_deposit())
                .with_static_gas(POA_TOKEN_FT_MINT_GAS)
                .ft_mint(CURRENT_ACCOUNT_ID.clone(), amount, None)
                .then(
                    ext_ft_core::ext(token_id)
                        .with_attached_deposit(NearToken::from_yoctonear(1))
                        // TODO: gas?
                        .ft_transfer_call(owner_id, amount, memo, msg),
                )
        } else {
            ext_poa_fungible_token::ext(token_id)
                .with_attached_deposit(env::attached_deposit())
                .with_static_gas(POA_TOKEN_FT_MINT_GAS)
                .ft_mint(owner_id, amount, memo)
        }
    }
}

impl POAFactoryImpl {
    fn token_id(token: impl AsRef<str>) -> Result<AccountId, ParseAccountError> {
        format!("{}.{}", token.as_ref(), *CURRENT_ACCOUNT_ID).parse()
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum Prefix {
    Tokens,
}
