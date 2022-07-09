use super::BInt;
use crate::{doc, BUint, ExpType};

#[doc=doc::saturating::impl_desc!()]
impl<const N: usize> BInt<N> {
    #[doc=doc::saturating::saturating_add!(I)]
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        match self.checked_add(rhs) {
            Some(add) => add,
            None => {
                if self.is_negative() {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    #[doc=doc::saturating::saturating_add_unsigned!(I)]
    #[inline]
    pub const fn saturating_add_unsigned(self, rhs: BUint<N>) -> Self {
        match self.checked_add_unsigned(rhs) {
            Some(i) => i,
            None => Self::MAX,
        }
    }

    #[doc=doc::saturating::saturating_sub!(I)]
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        match self.checked_sub(rhs) {
            Some(add) => add,
            None => {
                if self.is_negative() {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    #[doc=doc::saturating::saturating_sub_unsigned!(I)]
    #[inline]
    pub const fn saturating_sub_unsigned(self, rhs: BUint<N>) -> Self {
        match self.checked_sub_unsigned(rhs) {
            Some(i) => i,
            None => Self::MIN,
        }
    }

    crate::nightly::const_fns! {
        #[doc=doc::saturating::saturating_mul!(I)]
        #[inline]
        pub const fn saturating_mul(self, rhs: Self) -> Self {
            match self.checked_mul(rhs) {
                Some(mul) => mul,
                None => {
                    if self.is_negative() == rhs.is_negative() {
                        Self::MAX
                    } else {
                        Self::MIN
                    }
                }
            }
        }

        #[doc=doc::saturating::saturating_div!(I)]
        #[inline]
        pub const fn saturating_div(self, rhs: Self) -> Self {
            let (div, overflow) = self.overflowing_div(rhs);
            if overflow {
                Self::MAX
            } else {
                div
            }
        }

        #[doc=doc::saturating::saturating_neg!(I)]
        #[inline]
        pub const fn saturating_neg(self) -> Self {
            match self.checked_neg() {
                Some(abs) => abs,
                None => Self::MAX,
            }
        }

        #[doc=doc::saturating::saturating_abs!(I)]
        #[inline]
        pub const fn saturating_abs(self) -> Self {
            match self.checked_abs() {
                Some(abs) => abs,
                None => Self::MAX,
            }
        }

        #[doc=doc::saturating::saturating_pow!(I)]
        #[inline]
        pub const fn saturating_pow(self, exp: ExpType) -> Self {
            match self.checked_pow(exp) {
                Some(pow) => pow,
                None => {
                    if self.is_negative() && exp & 1 != 0 {
                        Self::MIN
                    } else {
                        Self::MAX
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::{test_bignum, types::*};

    test_bignum! {
        function: <itest>::saturating_add(a: itest, b: itest)
    }
    test_bignum! {
        function: <itest>::saturating_add_unsigned(a: itest, b: utest)
    }
    test_bignum! {
        function: <itest>::saturating_sub(a: itest, b: itest)
    }
    test_bignum! {
        function: <itest>::saturating_sub_unsigned(a: itest, b: utest)
    }
    test_bignum! {
        function: <itest>::saturating_neg(a: itest),
        cases: [
            (itest::MIN)
        ]
    }
    test_bignum! {
        function: <itest>::saturating_abs(a: itest),
        cases: [
            (itest::MIN)
        ]
    }
    test_bignum! {
        function: <itest>::saturating_mul(a: itest, b: itest)
    }
    test_bignum! {
        function: <itest>::saturating_pow(a: itest, b: u16)
    }
}