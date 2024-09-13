use near_sdk::{borsh::BorshSerialize, BorshStorageKey, IntoStorageKey};

pub trait NestPrefix: Sized + IntoStorageKey {
    fn nest<S>(self, nested: S) -> NestedPrefix<Self, S> {
        NestedPrefix {
            parent: self,
            nested,
        }
    }
}
impl<T> NestPrefix for T where T: IntoStorageKey {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
pub struct NestedPrefix<S, P> {
    parent: S,
    nested: P,
}
