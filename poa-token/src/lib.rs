use defuse_contracts::{
    poa::token::{POAFungibleToken, WITHDRAW_MEMO_PREFIX},
    utils::cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
};
use near_contract_standards::{
    fungible_token::{
        events::{FtBurn, FtMint},
        metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
        FungibleToken, FungibleTokenCore, FungibleTokenResolver,
    },
    storage_management::StorageManagement,
};
use near_plugins::{only, Ownable};
use near_sdk::{
    assert_one_yocto, borsh::BorshSerialize, json_types::U128, near, require, store::Lazy,
    AccountId, BorshStorageKey, PanicOnDefault, PromiseOrValue,
};

#[near(contract_state)]
#[derive(Ownable, PanicOnDefault)]
pub struct POAFungibleTokenImpl {
    token: FungibleToken,
    metadata: Lazy<FungibleTokenMetadata>,
}

#[near]
impl POAFungibleTokenImpl {
    #[init]
    pub fn new(owner_id: Option<AccountId>, metadata: FungibleTokenMetadata) -> Self {
        metadata.assert_valid();
        let mut contract = Self {
            token: FungibleToken::new(Prefix::FungibleToken),
            metadata: Lazy::new(Prefix::Metadata, metadata),
        };
        contract.owner_set(Some(
            owner_id.unwrap_or_else(|| PREDECESSOR_ACCOUNT_ID.clone()),
        ));
        contract
    }
}

#[near]
impl POAFungibleToken for POAFungibleTokenImpl {
    #[only(owner)]
    #[payable]
    fn ft_mint(&mut self, owner_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.storage_deposit(Some(owner_id.clone()), None);
        self.token.internal_deposit(&owner_id, amount.into());
        FtMint {
            owner_id: &owner_id,
            amount,
            memo: memo.as_deref(),
        }
        .emit();
    }
}

#[near]
impl FungibleTokenCore for POAFungibleTokenImpl {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        if receiver_id == *CURRENT_ACCOUNT_ID
            && memo
                .as_deref()
                // TODO: check if non-empty after prefix?
                .map_or(false, |memo| memo.starts_with(WITHDRAW_MEMO_PREFIX))
        {
            self.ft_withdraw(&PREDECESSOR_ACCOUNT_ID, amount, memo);
        } else {
            self.token.ft_transfer(receiver_id, amount, memo)
        }
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // TODO: storage management?
        // TODO: check if receiver self and withdraw msg
        // self.token.internal_deposit(account_id, amount);
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenResolver for POAFungibleTokenImpl {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        // TODO
        self.token
            .ft_resolve_transfer(sender_id, receiver_id, amount)
    }
}

#[near]
impl FungibleTokenMetadataProvider for POAFungibleTokenImpl {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.clone()
    }
}

impl POAFungibleTokenImpl {
    fn ft_withdraw(&mut self, account_id: &AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        require!(amount.0 > 0, "The amount should be a positive number");
        self.token.internal_withdraw(account_id, amount.into());
        FtBurn {
            owner_id: account_id,
            amount,
            memo: memo.as_deref(),
        }
        .emit();
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum Prefix {
    FungibleToken,
    Metadata,
}
