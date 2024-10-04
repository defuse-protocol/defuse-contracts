pub trait MulDiv<RHS = Self>: Sized {
    fn mul_div(self, mul: RHS, div: RHS) -> Option<Self>;
    fn mul_div_ceil(self, mul: RHS, div: RHS) -> Option<Self>;
}

impl MulDiv for u128 {
    fn mul_div(self, mul: Self, div: Self) -> Option<Self> {
        let (res, overflowed) = self.overflowing_mul(mul);
        if !overflowed {
            return res.checked_div(div);
        }
        todo!()
    }

    fn mul_div_ceil(self, mul: Self, div: Self) -> Option<Self> {
        todo!()
    }
}
