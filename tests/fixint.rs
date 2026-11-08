#![cfg(all(feature = "derive", feature = "alloc"))]

use bincode::config::{self, Config};
use bincode::error::DecodeError;

#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug)]
struct Hashed {
    #[bincode(fixint)]
    hash: u64,
    len: u32,
}

#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug)]
struct TupleHashed(#[bincode(fixint)] u32, u32);

#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug)]
enum HashedEnum {
    Named {
        #[bincode(fixint)]
        hash: u64,
        len: u32,
    },
    Tuple(#[bincode(fixint)] u16, u16),
}

/// `BorrowDecode` is derived instead of `Decode` because of the lifetime.
#[derive(bincode::Encode, bincode::BorrowDecode, PartialEq, Debug)]
struct Borrowed<'a> {
    name: &'a str,
    #[bincode(fixint)]
    hash: u128,
}

/// The attribute applies to everything inside the field, not only to bare integers.
#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug)]
struct Nested {
    #[bincode(fixint)]
    array: [u32; 2],
    #[bincode(fixint)]
    option: Option<u64>,
}

#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug)]
struct WithAndFixint {
    #[bincode(with = "encode_as_double", fixint)]
    value: u32,
}

mod encode_as_double {
    pub fn encode<E: bincode::enc::Encoder>(
        value: &u32,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&(value * 2), encoder)
    }
    pub fn decode<Context, D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<u32, bincode::error::DecodeError> {
        Ok(<u32 as bincode::Decode<Context>>::decode(decoder)? / 2)
    }
    pub fn borrow_decode<'de, Context, D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<u32, bincode::error::DecodeError> {
        Ok(<u32 as bincode::BorrowDecode<'de, Context>>::borrow_decode(decoder)? / 2)
    }
}

fn roundtrip<T, C>(value: &T, config: C) -> Vec<u8>
where
    T: bincode::Encode + bincode::Decode<()> + PartialEq + core::fmt::Debug,
    C: Config,
{
    let bytes = bincode::encode_to_vec(value, config).unwrap();
    let (decoded, len): (T, usize) = bincode::decode_from_slice(&bytes, config).unwrap();
    assert_eq!(&decoded, value);
    assert_eq!(len, bytes.len());
    bytes
}

#[test]
fn test_fixint_struct() {
    let value = Hashed {
        hash: 0x0102_0304_0506_0708,
        len: 5,
    };
    // The hash is written as 8 fixed bytes, the length as a 1-byte varint
    assert_eq!(
        roundtrip(&value, config::standard()),
        [8, 7, 6, 5, 4, 3, 2, 1, 5]
    );
    // Endianness from the config is preserved
    assert_eq!(
        roundtrip(&value, config::standard().with_big_endian()),
        [1, 2, 3, 4, 5, 6, 7, 8, 5]
    );
    // When the config already uses fixint, the attribute changes nothing
    assert_eq!(
        roundtrip(&value, config::legacy()),
        [8, 7, 6, 5, 4, 3, 2, 1, 5, 0, 0, 0]
    );

    // Small values still take the full width
    let small = Hashed { hash: 1, len: 1 };
    assert_eq!(
        roundtrip(&small, config::standard()),
        [1, 0, 0, 0, 0, 0, 0, 0, 1]
    );
}

#[test]
fn test_fixint_tuple_struct() {
    assert_eq!(
        roundtrip(&TupleHashed(1, 2), config::standard()),
        [1, 0, 0, 0, 2]
    );
}

#[test]
fn test_fixint_enum() {
    assert_eq!(
        roundtrip(&HashedEnum::Named { hash: 1, len: 2 }, config::standard()),
        // variant index (varint), hash (fixint), len (varint)
        [0, 1, 0, 0, 0, 0, 0, 0, 0, 2]
    );
    assert_eq!(
        roundtrip(&HashedEnum::Tuple(1, 2), config::standard()),
        [1, 1, 0, 2]
    );
}

#[test]
fn test_fixint_borrow_decode() {
    let value = Borrowed {
        name: "abc",
        hash: 7,
    };
    let bytes = bincode::encode_to_vec(&value, config::standard()).unwrap();
    assert_eq!(bytes.len(), 1 + 3 + 16);
    let (decoded, len): (Borrowed, usize) =
        bincode::borrow_decode_from_slice(&bytes, config::standard()).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(len, bytes.len());
}

#[test]
fn test_fixint_nested() {
    let value = Nested {
        array: [1, 2],
        option: Some(3),
    };
    assert_eq!(
        roundtrip(&value, config::standard()),
        [1, 0, 0, 0, 2, 0, 0, 0, 1, 3, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn test_fixint_with() {
    assert_eq!(
        roundtrip(&WithAndFixint { value: 3 }, config::standard()),
        [6, 0, 0, 0]
    );
}

#[test]
fn test_fixint_respects_limit() {
    let value = Hashed { hash: 1, len: 1 };
    let bytes = bincode::encode_to_vec(&value, config::standard()).unwrap();
    let result: Result<(Hashed, usize), _> =
        bincode::decode_from_slice(&bytes, config::standard().with_limit::<4>());
    assert!(matches!(result, Err(DecodeError::LimitExceeded)));
}
