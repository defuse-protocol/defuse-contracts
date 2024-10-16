//! Helper for [`serde_with::base64::Base64`] to implement [`serde_with::schemars_0_8::JsonSchemaAs`] on it.

use near_sdk::serde::{Deserializer, Serializer};
use serde_with::{
    base64::{Alphabet, Standard},
    formats::{Format, Padded, Unpadded},
    DeserializeAs, SerializeAs,
};

pub struct Base64<ALPHABET: Alphabet = Standard, PADDING: Format = Padded>(
    ::serde_with::base64::Base64<ALPHABET, PADDING>,
);

impl<T, ALPHABET> SerializeAs<T> for Base64<ALPHABET, Padded>
where
    T: AsRef<[u8]>,
    ALPHABET: Alphabet,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ::serde_with::base64::Base64::<ALPHABET, Padded>::serialize_as(source, serializer)
    }
}

impl<T, ALPHABET> SerializeAs<T> for Base64<ALPHABET, Unpadded>
where
    T: AsRef<[u8]>,
    ALPHABET: Alphabet,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ::serde_with::base64::Base64::<ALPHABET, Unpadded>::serialize_as(source, serializer)
    }
}

impl<'de, T, ALPHABET, FORMAT> DeserializeAs<'de, T> for Base64<ALPHABET, FORMAT>
where
    T: TryFrom<Vec<u8>>,
    ALPHABET: Alphabet,
    FORMAT: Format,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        ::serde_with::base64::Base64::<ALPHABET, FORMAT>::deserialize_as(deserializer)
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::schemars::{
        gen::SchemaGenerator,
        schema::{InstanceType, Schema, SchemaObject},
        JsonSchema,
    };
    use serde_with::schemars_0_8::JsonSchemaAs;

    impl<T, ALPHABET, FORMAT> JsonSchemaAs<T> for Base64<ALPHABET, FORMAT>
    where
        ALPHABET: Alphabet,
        FORMAT: Format,
    {
        fn schema_name() -> String {
            String::schema_name()
        }

        fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
            // TODO: use #[schemars(extend(...))] when released
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                extensions: [("contentEncoding", "base64".into())]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
                ..Default::default()
            }
            .into()
        }

        fn is_referenceable() -> bool {
            false
        }
    }
}
