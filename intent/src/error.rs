#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ContractError {
    BorshSerializeError,
    BorshDeserializeError,
    Base64EncodeError,
    Base64DecodeError,
}

impl AsRef<str> for ContractError {
    fn as_ref(&self) -> &str {
        match self {
            Self::BorshSerializeError => "BORSH_SERIALIZE_ERROR",
            Self::BorshDeserializeError => "BORSH_DESERIALIZE_ERROR",
            Self::Base64EncodeError => "BASE64_ENCODE_ERROR",
            Self::Base64DecodeError => "BASE64_DECODE_ERROR",
        }
    }
}
