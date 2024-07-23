use near_sdk::near;

#[derive(Debug, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
pub struct Mutex<T> {
    #[serde(flatten)]
    value: T,
    #[serde(
        default,
        // do not serialize `false`
        skip_serializing_if = "::core::ops::Not::not"
    )]
    locked: bool,
}

impl<T> Mutex<T> {
    #[must_use]
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            locked: false,
        }
    }

    #[inline]
    pub const fn as_locked(&self) -> Option<&T> {
        if !self.locked {
            return None;
        }
        Some(&self.value)
    }

    #[inline]
    pub fn as_locked_mut(&mut self) -> Option<&mut T> {
        if !self.locked {
            return None;
        }
        Some(&mut self.value)
    }

    #[inline]
    pub fn lock(&mut self) -> Option<&mut T> {
        if self.locked {
            return None;
        }
        self.locked = true;
        Some(&mut self.value)
    }

    #[inline]
    pub const fn as_unlocked(&self) -> Option<&T> {
        if self.locked {
            return None;
        }
        Some(&self.value)
    }

    #[inline]
    pub fn as_unlocked_mut(&mut self) -> Option<&mut T> {
        if self.locked {
            return None;
        }
        Some(&mut self.value)
    }

    #[inline]
    pub fn unlock(&mut self) -> Option<&mut T> {
        if !self.locked {
            return None;
        }
        self.locked = false;
        Some(&mut self.value)
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
