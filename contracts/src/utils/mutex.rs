use near_sdk::near;

#[near(serializers = [borsh])]
pub struct Mutex<T> {
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
    pub fn unlock(&mut self) -> bool {
        if !self.locked {
            return false;
        }
        self.locked = false;
        true
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
