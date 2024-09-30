use std::fmt::Display;

use near_sdk::{env, FunctionError};

pub trait PanicError {
    #[inline]
    fn panic_str(&self) -> !
    where
        Self: AsRef<str>,
    {
        env::panic_str(self.as_ref())
    }

    #[inline]
    fn panic_static_str(self) -> !
    where
        Self: Into<&'static str>,
    {
        self.into().panic_str()
    }

    #[inline]
    fn panic_display(&self) -> !
    where
        Self: Display,
    {
        self.to_string().panic_str()
    }
}
impl<E> PanicError for E {}

pub trait UnwrapOrPanic<T, E> {
    fn unwrap_or_panic(self) -> T
    where
        E: FunctionError;

    fn unwrap_or_panic_str(self) -> T
    where
        E: AsRef<str>;

    fn unwrap_or_panic_static_str(self) -> T
    where
        E: Into<&'static str>;

    fn unwrap_or_panic_display(self) -> T
    where
        E: Display;
}

impl<T, E> UnwrapOrPanic<T, E> for Result<T, E> {
    #[inline]
    fn unwrap_or_panic(self) -> T
    where
        E: FunctionError,
    {
        self.unwrap_or_else(|err| err.panic())
    }

    #[inline]
    fn unwrap_or_panic_str(self) -> T
    where
        E: AsRef<str>,
    {
        self.unwrap_or_else(|err| err.panic_str())
    }

    #[inline]
    fn unwrap_or_panic_static_str(self) -> T
    where
        E: Into<&'static str>,
    {
        self.unwrap_or_else(|err| err.panic_static_str())
    }

    #[inline]
    fn unwrap_or_panic_display(self) -> T
    where
        E: Display,
    {
        self.unwrap_or_else(|err| err.panic_display())
    }
}
