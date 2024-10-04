use impl_tools::autoimpl;
use near_sdk::{near, AccountId};

use crate::{crypto::SignedPayload, utils::Deadline};

use super::payload::MultiStandardPayload;

pub type SignedDefuseMessage<T> = SignedPayload<MultiStandardPayload<DefuseMessage<T>>>;

#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.message)]
#[autoimpl(DerefMut using self.message)]
#[derive(Debug, Clone)]
pub struct DefuseMessage<T> {
    pub signer_id: AccountId,

    pub deadline: Deadline,

    #[serde(flatten)]
    pub message: T,
}
