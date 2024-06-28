use defuse_contracts::account::AccountContract;
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::{env, near, AccountId, BorshStorageKey, PanicOnDefault};

mod nft;

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum Prefix {
    OwnerById,
    Metadata,
    Enumeration,
    Approvals,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AccountContractImpl {
    accounts: NonFungibleToken,
    // TODO: MPC
}

#[near]
impl AccountContractImpl {
    #[init]
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new(owner: AccountId) -> Self {
        Self {
            accounts: NonFungibleToken::new::<_, u8, u8, u8>(
                Prefix::OwnerById,
                owner,
                None,
                None,
                None,
            ),
        }
    }
}

#[near]
impl AccountContract for AccountContractImpl {
    #[payable]
    // TODO: make owner optional and default to env::predecessor_account_id()?
    // TODO: allow to create only accounts where owner is specified somewhere
    // in derivation_path. So, we should agree on a format
    fn create_account(&mut self, derivation_path: String, owner: Option<AccountId>) {
        self.accounts.internal_mint(
            derivation_path,
            owner.unwrap_or_else(env::predecessor_account_id),
            None,
        );
    }

    // TODO: storage management
}
