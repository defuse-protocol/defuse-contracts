use near_sdk::near;

#[derive(Debug, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
pub struct Mutex<T> {
    #[serde(flatten)]
    value: T,
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
    pub const fn get_unlocked(&self) -> Option<&T> {
        if self.locked {
            return None;
        }
        Some(&self.value)
    }

    #[inline]
    pub const fn get_locked(&self) -> Option<&T> {
        if !self.locked {
            return None;
        }
        Some(&self.value)
    }

    #[inline]
    pub fn get_unlocked_mut(&mut self) -> Option<&mut T> {
        if self.locked {
            return None;
        }
        Some(&mut self.value)
    }

    #[inline]
    pub fn get_locked_mut(&mut self) -> Option<&mut T> {
        if !self.locked {
            return None;
        }
        Some(&mut self.value)
    }

    #[inline]
    pub const unsafe fn get_unchecked(&self) -> &T {
        &self.value
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        &mut self.value
    }

    #[inline]
    pub fn lock(&mut self) -> Option<&T> {
        if self.locked {
            return None;
        }
        self.locked = true;
        Some(&self.value)
    }

    #[inline]
    pub fn lock_mut(&mut self) -> Option<&mut T> {
        if self.locked {
            return None;
        }
        self.locked = true;
        Some(&mut self.value)
    }

    #[inline]
    pub fn unlock(&mut self) -> Option<&T> {
        if !self.locked {
            return None;
        }
        self.locked = false;
        Some(&self.value)
    }

    #[inline]
    pub fn unlock_mut(&mut self) -> Option<&mut T> {
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
