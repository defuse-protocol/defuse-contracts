use near_sdk::{near, BorshStorageKey, IntoStorageKey};

pub trait NestPrefix: Sized + IntoStorageKey {
    fn nest<S>(self, nested: S) -> NestedPrefix<Self, S> {
        NestedPrefix {
            parent: self,
            nested,
        }
    }
}
impl<T> NestPrefix for T where T: IntoStorageKey {}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
pub struct NestedPrefix<S, P> {
    parent: S,
    nested: P,
}
