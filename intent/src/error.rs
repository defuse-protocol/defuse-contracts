use near_sdk::base64;
use strum::IntoStaticStr;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractError {
    #[strum(serialize = "BORSH_SERIALIZE_ERROR")]
    BorshSerialize,
    #[strum(serialize = "BORSH_DESERIALIZE_ERROR")]
    BorshDeserialize,
    #[strum(serialize = "BASE64_DECODE_ERROR")]
    Base64Decode,
}

impl From<base64::DecodeError> for ContractError {
    fn from(_: base64::DecodeError) -> Self {
        Self::Base64Decode
    }
}
