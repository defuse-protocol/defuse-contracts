use near_sdk::near;

#[derive(Default, Clone)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
#[near(serializers=[borsh, json])]
pub struct Account {
    is_locked: bool,
}
