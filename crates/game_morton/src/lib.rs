use bitintr::{Pdep, Pext};

/// Provides functions for encoding/decoding coordinates using Morton z-curve encoding.
/// This encoding attempts to ensure that spatial proximity translates to proximity
/// within memory when stored in a single dimensional array by interleaving the bits.
///
/// # Remarks
/// Each of the encode/decode functions is an associated function.
///
/// # Examples
/// ```
/// use game_morton::Morton;
///
/// let encoded = Morton::encode_2d(0b101u32, 0b011u32);
/// assert_eq!(encoded, 0b011011u64);
/// ```
pub trait Morton: Sized {
    type Encoded;

    fn encode_2d(x: Self, y: Self) -> Self::Encoded;
    fn decode_2d(z: Self::Encoded) -> (Self, Self);
}

macro_rules! impl_morton {
    ($dec:ty => $enc:ty [$x_mask:expr, $y_mask:expr]) => {
        // Based on https://stackoverflow.com/a/30540867/8430206
        impl Morton for $dec {
            type Encoded = $enc;

            fn encode_2d(x: Self, y: Self) -> Self::Encoded {
                let x = x as Self::Encoded;
                let y = y as Self::Encoded;
                x.pdep($x_mask) | y.pdep($y_mask)
            }

            fn decode_2d(z: Self::Encoded) -> (Self, Self) {
                let x = z.pext($x_mask) as Self;
                let y = z.pext($y_mask) as Self;
                (x, y)
            }
        }
    };
}

impl_morton!(u32 => u64 [0x55555555_55555555, 0xAAAAAAAA_AAAAAAAA]);
impl_morton!(u16 => u32 [0x55555555, 0xAAAAAAAA]);
impl_morton!(u8 => u16 [0x5555, 0xAAAA]);

impl_morton!(i32 => i64 [0x55555555_55555555, -0x55555555_55555556]);
impl_morton!(i16 => i32 [0x55555555, -0x55555556]);
impl_morton!(i8 => i16 [0x5555, -0x5556]);
