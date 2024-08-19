use near_sdk::{near, AccountId};

#[near(serializers = [json])]
pub struct Approval {
    pub owner_id: AccountId,
    pub approval_id: u64,
}
