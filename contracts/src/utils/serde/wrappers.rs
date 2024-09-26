use core::{
    fmt::{self, Display},
    str::FromStr,
};

use serde_with::{DeserializeFromStr, SerializeDisplay};

/// Helper type to implement `#[derive(Serialize, Deserialize)]`,
/// as `#[near_bindgen]` doesn't support `#[serde(...)]` attributes on method arguments
#[derive(Debug, Clone, Copy, Default, SerializeDisplay, DeserializeFromStr)]
pub struct DisplayFromStr<T>(pub T);

impl<T> DisplayFromStr<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Display for DisplayFromStr<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> FromStr for DisplayFromStr<T>
where
    T: FromStr,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(Self)
    }
}

impl<T> From<T> for DisplayFromStr<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

    impl<T> JsonSchema for DisplayFromStr<T>
    where
        T: Display + FromStr<Err: Display>,
    {
        fn schema_name() -> String {
            String::schema_name()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }

        fn is_referenceable() -> bool {
            false
        }
    }
}
