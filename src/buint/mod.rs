use crate::digit::{self, Digit, DoubleDigit};

#[cfg(debug_assertions)]
use crate::errors::{self, option_expect};

use crate::doc;
use crate::nightly::const_fn;
use crate::ExpType;
use core::mem::MaybeUninit;

#[inline]
pub const fn carrying_mul(a: Digit, b: Digit, carry: Digit, current: Digit) -> (Digit, Digit) {
    let prod =
        carry as DoubleDigit + current as DoubleDigit + (a as DoubleDigit) * (b as DoubleDigit);
    (prod as Digit, (prod >> Digit::BITS) as Digit)
}

const_fn! {
    #[inline]
    pub const unsafe fn unchecked_shl<const N: usize>(u: BUint<N>, rhs: ExpType) -> BUint<N> {
        let mut out = BUint::ZERO;
        let digit_shift = (rhs >> digit::BIT_SHIFT) as usize;
        let bit_shift = rhs & digit::BITS_MINUS_1;

        let num_copies = N - digit_shift;

        u.digits.as_ptr().copy_to_nonoverlapping(out.digits.as_mut_ptr().add(digit_shift), num_copies);

        if bit_shift != 0 {
            let carry_shift = digit::BITS - bit_shift;
            let mut carry = 0;

            let mut i = digit_shift;
            while i < N {
                let current_digit = out.digits[i];
                out.digits[i] = (current_digit << bit_shift) | carry;
                carry = current_digit >> carry_shift;
                i += 1;
            }
        }

        out
    }
}

const_fn! {
    #[inline]
    pub const unsafe fn unchecked_shr_pad<const N: usize, const PAD: Digit>(u: BUint<N>, rhs: ExpType) -> BUint<N> {
        let mut out = BUint::from_digits([PAD; N]);
        let digit_shift = (rhs >> digit::BIT_SHIFT) as usize;
        let bit_shift = rhs & digit::BITS_MINUS_1;

        let num_copies = N - digit_shift;

        u.digits.as_ptr().add(digit_shift).copy_to_nonoverlapping(out.digits.as_mut_ptr(), num_copies);

        if bit_shift != 0 {
            let carry_shift = digit::BITS - bit_shift;
            let mut carry = 0;

            let mut i = num_copies;
            while i > 0 {
                i -= 1;
                let current_digit = out.digits[i];
                out.digits[i] = (current_digit >> bit_shift) | carry;
                carry = current_digit << carry_shift;
            }

            if PAD == Digit::MAX {
                out.digits[num_copies - 1] |= Digit::MAX << carry_shift;
            }
        }

        out
    }
}

const_fn! {
    pub const unsafe fn unchecked_shr<const N: usize>(u: BUint<N>, rhs: ExpType) -> BUint<N> {
        unchecked_shr_pad::<N, {Digit::MIN}>(u, rhs)
    }
}

#[cfg(feature = "serde")]
use ::{
    serde::{Deserialize, Serialize},
    serde_big_array::BigArray,
};

/// Big unsigned integer type, of fixed size which must be known at compile time.
///
/// Digits are stored in little endian (least significant digit first). `BUint<N>` aims to exactly replicate the behaviours of Rust's built-in unsigned integer types: `u8`, `u16`, `u32`, `u64`, `u128` and `usize`. The const generic parameter `N` is the number of digits that are stored.

// Clippy: we can allow derivation of `Hash` and manual implementation of `PartialEq` as the derived `PartialEq` would be the same except we make our implementation const.
#[allow(clippy::derive_hash_xor_eq)]
#[derive(Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BUint<const N: usize> {
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    pub(crate) digits: [Digit; N],
}

mod consts;

impl<const N: usize> BUint<N> {
    #[doc=doc::count_ones!(U 1024)]
    #[inline]
    pub const fn count_ones(self) -> ExpType {
        let mut ones = 0;
        let mut i = 0;
        while i < N {
            ones += self.digits[i].count_ones() as ExpType;
            i += 1;
        }
        ones
    }

    #[doc=doc::count_zeros!(U 1024)]
    #[inline]
    pub const fn count_zeros(self) -> ExpType {
        let mut zeros = 0;
        let mut i = 0;
        while i < N {
            zeros += self.digits[i].count_zeros() as ExpType;
            i += 1;
        }
        zeros
    }

    #[doc=doc::leading_zeros!(U 1024)]
    #[inline]
    pub const fn leading_zeros(self) -> ExpType {
        let mut zeros = 0;
        let mut i = N;
        while i > 0 {
            i -= 1;
            let digit = self.digits[i];
            zeros += digit.leading_zeros() as ExpType;
            if digit != Digit::MIN {
                break;
            }
        }
        zeros
    }

    #[doc=doc::trailing_zeros!(U 1024)]
    #[inline]
    pub const fn trailing_zeros(self) -> ExpType {
        let mut zeros = 0;
        let mut i = 0;
        while i < N {
            let digit = self.digits[i];
            zeros += digit.trailing_zeros() as ExpType;
            if digit != Digit::MIN {
                break;
            }
            i += 1;
        }
        zeros
    }

    #[doc=doc::leading_ones!(U 1024, MAX)]
    #[inline]
    pub const fn leading_ones(self) -> ExpType {
        let mut ones = 0;
        let mut i = N;
        while i > 0 {
            i -= 1;
            let digit = self.digits[i];
            ones += digit.leading_ones() as ExpType;
            if digit != Digit::MAX {
                break;
            }
        }
        ones
    }

    #[doc=doc::trailing_ones!(U 1024)]
    #[inline]
    pub const fn trailing_ones(self) -> ExpType {
        let mut ones = 0;
        let mut i = 0;
        while i < N {
            let digit = self.digits[i];
            ones += digit.trailing_ones() as ExpType;
            if digit != Digit::MAX {
                break;
            }
            i += 1;
        }
        ones
    }

    crate::nightly::const_fns! {
        #[inline]
        const unsafe fn rotate_digits_left(self, n: usize) -> Self {
            let mut uninit = MaybeUninit::<[Digit; N]>::uninit();
            let digits_ptr = self.digits.as_ptr();
            let uninit_ptr = uninit.as_mut_ptr() as *mut Digit;

            digits_ptr.copy_to_nonoverlapping(uninit_ptr.add(n), N - n);
            digits_ptr.add(N - n).copy_to_nonoverlapping(uninit_ptr, n);
            Self::from_digits(uninit.assume_init())
        }

        #[inline]
        const unsafe fn unchecked_rotate_left(self, rhs: ExpType) -> Self {
            let digit_shift = (rhs >> digit::BIT_SHIFT) as usize;
            let bit_shift = rhs & digit::BITS_MINUS_1;

            let mut out = self.rotate_digits_left(digit_shift);

            if bit_shift != 0 {
                let carry_shift = digit::BITS - bit_shift;
                let mut carry = 0;

                let mut i = 0;
                while i < N {
                    let current_digit = out.digits[i];
                    out.digits[i] = (current_digit << bit_shift) | carry;
                    carry = current_digit >> carry_shift;
                    i += 1;
                }
                out.digits[0] |= carry;
            }

            out
        }
    }

    const BITS_MINUS_1: ExpType = (Self::BITS - 1) as ExpType;

    crate::nightly::const_fns! {
        #[doc=doc::rotate_left!(U 256, "u")]
        #[inline]
        pub const fn rotate_left(self, n: ExpType) -> Self {
            unsafe {
                self.unchecked_rotate_left(n & Self::BITS_MINUS_1)
            }
        }

        #[doc=doc::rotate_right!(U 256, "u")]
        #[inline]
        pub const fn rotate_right(self, n: ExpType) -> Self {
            let n = n & Self::BITS_MINUS_1;
            unsafe {
                self.unchecked_rotate_left(Self::BITS as ExpType - n)
            }
        }
    }

    const N_MINUS_1: usize = N - 1;

    #[doc=doc::swap_bytes!(U 256, "u")]
    #[inline]
    pub const fn swap_bytes(self) -> Self {
        let mut uint = Self::ZERO;
        let mut i = 0;
        while i < N {
            uint.digits[i] = self.digits[Self::N_MINUS_1 - i].swap_bytes();
            i += 1;
        }
        uint
    }

    #[doc=doc::reverse_bits!(U 256, "u")]
    #[inline]
    pub const fn reverse_bits(self) -> Self {
        let mut uint = Self::ZERO;
        let mut i = 0;
        while i < N {
            uint.digits[i] = self.digits[Self::N_MINUS_1 - i].reverse_bits();
            i += 1;
        }
        uint
    }

    #[doc=doc::pow!(U 256)]
    #[inline]
    pub const fn pow(self, exp: ExpType) -> Self {
        #[cfg(debug_assertions)]
        return option_expect!(
            self.checked_pow(exp),
            errors::err_msg!("attempt to calculate power with overflow")
        );
        #[cfg(not(debug_assertions))]
        self.wrapping_pow(exp)
    }

    crate::nightly::const_fns! {
        #[doc=doc::div_euclid!(U)]
        #[inline]
        pub const fn div_euclid(self, rhs: Self) -> Self {
            self.wrapping_div_euclid(rhs)
        }


        #[doc=doc::rem_euclid!(U)]
        #[inline]
        pub const fn rem_euclid(self, rhs: Self) -> Self {
            self.wrapping_rem_euclid(rhs)
        }
    }

    #[doc=doc::doc_comment! {
        U 256,
        "Returns `true` if and only if `self == 2^k` for some integer `k`.",

        "let n = " stringify!(U256) "::from(1u16 << 14);\n"
        "assert!(n.is_power_of_two());\n"
        "let m = " stringify!(U256) "::from(100u8);\n"
        "assert!(!m.is_power_of_two());"
    }]
    #[inline]
    pub const fn is_power_of_two(&self) -> bool {
        let mut i = 0;
        let mut ones = 0;
        while i < N {
            ones += (&self.digits)[i].count_ones();
            if ones > 1 {
                return false;
            }
            i += 1;
        }
        ones == 1
    }

    #[doc=doc::next_power_of_two!(U 256, "0", "ZERO")]
    #[inline]
    pub const fn next_power_of_two(self) -> Self {
        #[cfg(debug_assertions)]
        return option_expect!(
            self.checked_next_power_of_two(),
            errors::err_msg!("attempt to calculate next power of two with overflow")
        );
        #[cfg(not(debug_assertions))]
        self.wrapping_next_power_of_two()
    }

    #[doc=doc::log2!(U)]
    #[inline]
    pub const fn log2(self) -> ExpType {
        #[cfg(debug_assertions)]
        return option_expect!(
            self.checked_log2(),
            errors::err_msg!("attempt to calculate log2 of zero")
        );
        #[cfg(not(debug_assertions))]
        match self.checked_log2() {
            Some(n) => n,
            None => 0,
        }
    }

    crate::nightly::const_fns! {
        #[doc=doc::log10!(U)]
        #[inline]
        pub const fn log10(self) -> ExpType {
            #[cfg(debug_assertions)]
            return option_expect!(self.checked_log10(), errors::err_msg!("attempt to calculate log10 of zero"));
            #[cfg(not(debug_assertions))]
            match self.checked_log10() {
                Some(n) => n,
                None => 0,
            }
        }

        #[doc=doc::log!(U)]
        #[inline]
        pub const fn log(self, base: Self) -> ExpType {
            #[cfg(debug_assertions)]
            return option_expect!(self.checked_log(base), errors::err_msg!("attempt to calculate log of zero or log with base < 2"));
            #[cfg(not(debug_assertions))]
            match self.checked_log(base) {
                Some(n) => n,
                None => 0,
            }
        }
    }

    crate::nightly::const_fns! {
        #[doc=doc::abs_diff!(U)]
        #[inline]
        pub const fn abs_diff(self, other: Self) -> Self {
            if self < other {
                other.wrapping_sub(self)
            } else {
                self.wrapping_sub(other)
            }
        }

        #[doc=doc::next_multiple_of!(U)]
        #[inline]
        pub const fn next_multiple_of(self, rhs: Self) -> Self {
            let rem = self.wrapping_rem(rhs);
            if rem.is_zero() {
                self
            } else {
                self + (rhs - rem)
            }
        }

        #[doc=doc::div_floor!(U)]
        #[inline]
        pub const fn div_floor(self, rhs: Self) -> Self {
            self.wrapping_div(rhs)
        }

        #[doc=doc::div_ceil!(U)]
        #[inline]
        pub const fn div_ceil(self, rhs: Self) -> Self {
            let (div, rem) = self.div_rem(rhs);
            if rem.is_zero() {
                div
            } else {
                div + Self::ONE
            }
        }
    }
}

impl<const N: usize> BUint<N> {
    #[doc=doc::bits!(U 256)]
    #[inline]
    pub const fn bits(&self) -> ExpType {
        Self::BITS as ExpType - self.leading_zeros()
    }

    #[doc=doc::bit!(U 256)]
    #[inline]
    pub const fn bit(&self, index: ExpType) -> bool {
        let digit = self.digits[index as usize >> digit::BIT_SHIFT];
        digit & (1 << (index & digit::BITS_MINUS_1)) != 0
    }

    /// Returns a `BUint` whose value is `2^power`.
    ///
    /// # Panics
    ///
    /// This function will panic if `power` is greater than or equal to `Self::BITS`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bnum::BUint;
    ///
    /// let power = 11;
    /// assert_eq!(BUint::<2>::power_of_two(11), (1u128 << 11).into());
    /// ```
    #[inline]
    pub const fn power_of_two(power: ExpType) -> Self {
        let mut out = Self::ZERO;
        out.digits[power as usize >> digit::BIT_SHIFT] = 1 << (power & (digit::BITS - 1));
        out
    }

    /// Returns the digits stored in `self` as an array. Digits are little endian (least significant digit first).
    #[inline(always)]
    pub const fn digits(&self) -> &[Digit; N] {
        &self.digits
    }

    /// Creates a new `BUint` from the given array of digits. Digits are stored as little endian (least significant digit first).
    #[inline(always)]
    pub const fn from_digits(digits: [Digit; N]) -> Self {
        Self { digits }
    }

    /// Creates a new `BUint` from the given digit. The given digit is stored as the least significant digit.
    #[inline(always)]
    pub const fn from_digit(digit: Digit) -> Self {
        let mut out = Self::ZERO;
        out.digits[0] = digit;
        out
    }

    #[doc=doc::is_zero!(U 256)]
    #[inline]
    pub const fn is_zero(&self) -> bool {
        let mut i = 0;
        while i < N {
            if (&self.digits)[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    #[doc=doc::is_one!(U 256)]
    #[inline]
    pub const fn is_one(&self) -> bool {
        if N == 0 || self.digits[0] != 1 {
            return false;
        }
        let mut i = 1;
        while i < N {
            if (&self.digits)[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    #[inline]
    const fn last_digit_index(&self) -> usize {
        let mut index = 0;
        let mut i = 1;
        while i < N {
            if (&self.digits)[i] != 0 {
                index = i;
            }
            i += 1;
        }
        index
    }

    #[inline]
    pub(crate) const fn to_exp_type(self) -> Option<ExpType> {
        let last_index = self.last_digit_index();
        if self.digits[last_index] == 0 {
            return Some(0);
        }
        if last_index >= ExpType::BITS as usize >> digit::BIT_SHIFT {
            return None;
        }
        let mut out = 0;
        let mut i = 0;
        while i <= last_index {
            out |= (self.digits[i] as ExpType) << (i << digit::BIT_SHIFT);
            i += 1;
        }
        Some(out)
    }

    #[allow(unused)]
    #[inline]
    fn square(self) -> Self {
        // TODO: optimise this method, this will make exponentiation by squaring faster
        self * self
    }
}

mod bigint_helpers;
mod cast;
mod checked;
mod cmp;
mod convert;
mod endian;
mod fmt;
#[cfg(feature = "numtraits")]
mod numtraits;
mod ops;
mod overflowing;
mod radix;
mod saturating;
mod unchecked;
mod wrapping;

use core::default::Default;

impl<const N: usize> Default for BUint<N> {
    #[doc=doc::default!()]
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

use core::iter::{Iterator, Product, Sum};

impl<const N: usize> Product<Self> for BUint<N> {
    #[inline]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |a, b| a * b)
    }
}

impl<'a, const N: usize> Product<&'a Self> for BUint<N> {
    #[inline]
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |a, b| a * b)
    }
}

impl<const N: usize> Sum<Self> for BUint<N> {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}

impl<'a, const N: usize> Sum<&'a Self> for BUint<N> {
    #[inline]
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}

#[cfg(test)]
mod tests {
    use crate::test::types::U128;
    use crate::test::{debug_skip, test_bignum, types::utest};

    crate::int::tests!(utest);

    test_bignum! {
        function: <utest>::next_power_of_two(a: utest),
        skip: debug_skip!(a.checked_next_power_of_two().is_none())
    }

    #[test]
    fn bit() {
        let u = U128::from(0b001010100101010101u128);
        assert!(u.bit(0));
        assert!(!u.bit(1));
        assert!(!u.bit(17));
        assert!(!u.bit(16));
        assert!(u.bit(15));
    }

    #[test]
    fn is_zero() {
        assert!(U128::MIN.is_zero());
        assert!(!U128::MAX.is_zero());
        assert!(!U128::ONE.is_zero());
    }

    #[test]
    fn is_one() {
        assert!(U128::ONE.is_one());
        assert!(!U128::MAX.is_one());
        assert!(!U128::ZERO.is_one());
    }

    #[test]
    fn bits() {
        let u = U128::from(0b1001010100101010101u128);
        assert_eq!(u.bits(), 19);

        let u = U128::power_of_two(78);
        assert_eq!(u.bits(), 79);
    }

    #[test]
    fn default() {
        assert_eq!(U128::default(), u128::default().into());
    }

    #[test]
    fn is_power_of_two() {
        let power = U128::from(1u128 << 88);
        let non_power = U128::from((1u128 << 88) - 5);
        assert!(power.is_power_of_two());
        assert!(!non_power.is_power_of_two());
    }
}