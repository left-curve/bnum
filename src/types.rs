//! Type aliases for big signed and unsigned integers. Each is an alias for either a [`BUint`] or a [`BInt`].

use crate::{errors::TryFromIntError, BInt, BTryFrom, BUint};

macro_rules! int_type_doc {
    ($bits: literal, $sign: literal) => {
        concat!($bits, "-bit ", $sign, " integer type.")
    };
}

macro_rules! int_types {
    { $($bits: literal $u: ident $i: ident; ) *}  => {
        $(
            #[doc = int_type_doc!($bits, "unsigned")]
            pub type $u = BUint::<{$bits / 64}>;

            #[doc = int_type_doc!($bits, "signed")]
            pub type $i = BInt::<{$bits / 64}>;
        )*
    };
}

macro_rules! call_types_macro {
    ($name: ident) => {
        $name! {
            128 U128 I128;
            256 U256 I256;
            512 U512 I512;
            1024 U1024 I1024;
            2048 U2048 I2048;
            4096 U4096 I4096;
            8192 U8192 I8192;
        }
    };
}

macro_rules! big_conversion {
    (
        $from:tt => $to:tt
    ) => {
        impl From<$from> for U512 {
            fn from(value: $from) -> Self {
                const FROM_BYTES_LEN: usize = <$from>::BYTES as usize;
                const TO_BYTES_LEN: usize = <$to>::BYTES as usize;

                // --- value.to_le_bytes() ---

                let words = value.digits();
                let mut bytes: [[u8; 8]; FROM_BYTES_LEN / 8] = [[0u8; 8]; FROM_BYTES_LEN / 8];
                for i in 0..FROM_BYTES_LEN / 8 {
                    bytes[i] = words[i].to_le_bytes();
                }

                let from_bytes: [u8; FROM_BYTES_LEN] = unsafe { core::mem::transmute(bytes) };
                let mut to_bytes = [0_u8; TO_BYTES_LEN];
                to_bytes[..FROM_BYTES_LEN].copy_from_slice(&from_bytes);

                // --- Value from le bytes ---

                let mut bytes = [0u64; TO_BYTES_LEN / 8];
                for i in 0..TO_BYTES_LEN / 8 {
                    bytes[i] = u64::from_le_bytes([
                        to_bytes[i * 8],
                        to_bytes[i * 8 + 1],
                        to_bytes[i * 8 + 2],
                        to_bytes[i * 8 + 3],
                        to_bytes[i * 8 + 4],
                        to_bytes[i * 8 + 5],
                        to_bytes[i * 8 + 6],
                        to_bytes[i * 8 + 7],
                    ])
                }
                Self::from_digits(bytes)
            }
        }

        impl TryFrom<$to> for $from {
            type Error = TryFromIntError;
            fn try_from(value: $to) -> Result<Self, Self::Error> {
                BTryFrom::<$to>::try_from(value)
            }
        }
    };
}

big_conversion!(U256 => U512);

call_types_macro!(int_types);

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! assert_int_bits {
        { $($bits: literal $u: ident $i: ident; ) *} => {
            $(
                assert_eq!($u::BITS, $bits);
                assert_eq!($i::BITS, $bits);
            )*
        }
    }

    #[test]
    fn test_int_bits() {
        call_types_macro!(assert_int_bits);
    }

    #[test]
    fn test_from_to() {
        let u256 = U256::from(42_u64);
        let u512: U512 = u256.into();
        assert_eq!(u512, U512::from(42_u64));

        let u256: U256 = TryFrom::<U512>::try_from(u512).unwrap();
        assert_eq!(u256, U256::from(42_u64));

        let u256: Result<U256, TryFromIntError> = TryFrom::<U512>::try_from(U512::MAX);
        assert!(u256.is_err());

        let u256 = U256::MAX;
        let mut u512: U512 = u256.into();
        let _: U256 = TryFrom::<U512>::try_from(u512).unwrap();

        u512 += U512::ONE;

        let u256: Result<U256, TryFromIntError> = TryFrom::<U512>::try_from(U512::MAX);
        assert!(u256.is_err());
    }
}
