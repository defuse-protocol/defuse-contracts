// use near_sdk::env;

#[derive(Debug)]
pub enum Error {
    AccountExist,
    EmptyAccounts,
    NoAccount,
}

impl AsRef<str> for Error {
    fn as_ref(&self) -> &str {
        match self {
            Self::AccountExist => "Account doesn't exist",
            Self::EmptyAccounts => "User doesn't have accounts",
            Self::NoAccount => "Account doesn't belong to the user",
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait LogError<T> {
    fn log_error(self) -> T;
}

impl<T, E> LogError<T> for Result<T, E>
where
    E: AsRef<str>,
{
    #[cfg(target_arch = "wasm32")]
    fn log_error(self) -> T {
        self.unwrap_or_else(|e| near_sdk::env::panic_str(e.as_ref()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn log_error(self) -> T {
        self.unwrap_or_else(|e| panic!("{}", e.as_ref()))
    }
}
