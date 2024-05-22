use near_workspaces::AccountId;

pub trait Account {
    async fn add_account(&self, account_id: AccountId);
}
