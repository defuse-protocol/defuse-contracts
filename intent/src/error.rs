use near_sdk::env;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ContractError {
    BorshSerializeError,
    BorshDeserializeError,
    Base64EncodeError,
    Base64DecodeError,
    IntentNotFound,
    IntentAlreadyExists,
    IntentExpired,
    IntentIsExecuting,
    WrongIntentStatus,
    IncorrectAmount,
}

impl AsRef<str> for ContractError {
    fn as_ref(&self) -> &str {
        match self {
            Self::BorshSerializeError => "BORSH_SERIALIZE_ERROR",
            Self::BorshDeserializeError => "BORSH_DESERIALIZE_ERROR",
            Self::Base64EncodeError => "BASE64_ENCODE_ERROR",
            Self::Base64DecodeError => "BASE64_DECODE_ERROR",
            Self::IntentNotFound => "INTENT_NOT_FOUND",
            Self::IntentAlreadyExists => "INTENT_ALREADY_EXISTS",
            Self::IntentExpired => "INTENT_EXPIRED",
            Self::IntentIsExecuting => "INTENT_IS_EXECUTING",
            Self::WrongIntentStatus => "WRONG_INTENT_STATUS",
            Self::IncorrectAmount => "INCORRECT_AMOUNT",
        }
    }
}

pub trait LogUnwrap<T> {
    fn log_unwrap(self) -> T;
}

impl<T, E> LogUnwrap<T> for Result<T, E>
where
    E: AsRef<str>,
{
    fn log_unwrap(self) -> T {
        self.unwrap_or_else(|e| env::panic_str(e.as_ref()))
    }
}
