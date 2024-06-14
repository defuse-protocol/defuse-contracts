use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("account doesn't exist")]
    AccountExist,
    #[error("user doesn't have any accounts")]
    EmptyAccounts,
    #[error("account doesn't belong to the user")]
    NoAccount,
}
