use strum::IntoStaticStr;

#[derive(Debug, IntoStaticStr)]
pub enum Error {
    #[strum(serialize = "account doesn't exist")]
    AccountExist,
    #[strum(serialize = "user doesn't have any accounts")]
    EmptyAccounts,
    #[strum(serialize = "account doesn't belong to the user")]
    NoAccount,
}

#[allow(clippy::module_name_repetitions)]
pub trait LogError<T> {
    fn log_error(self) -> T;
}

impl<T, E> LogError<T> for Result<T, E>
where
    E: Into<&'static str>,
{
    #[cfg(target_arch = "wasm32")]
    #[inline]
    fn log_error(self) -> T {
        self.map_err(Into::into)
            .unwrap_or_else(|e| near_sdk::env::panic_str(e))
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[inline]
    fn log_error(self) -> T {
        self.map_err(Into::into).unwrap_or_else(|e| panic!("{e}"))
    }
}
