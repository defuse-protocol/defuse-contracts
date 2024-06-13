use near_sdk::near;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[near(serializers=[borsh, json])]
pub struct Account {
    is_locked: bool,
}
