//! Contains serialization and deserialization code for types used throughout the system.
mod bytes;

use alloc::{
    alloc::{alloc, Layout},
    collections::{BTreeMap, BTreeSet, VecDeque},
    str,
    string::String,
    vec::Vec,
};
#[cfg(debug_assertions)]
use core::any;
use core::{mem, ptr::NonNull};

use num_integer::Integer;
use num_rational::Ratio;
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use thiserror::Error;

pub use bytes::Bytes;

/// The number of bytes in a serialized `()`.
pub const UNIT_SERIALIZED_LENGTH: usize = 0;
/// The number of bytes in a serialized `bool`.
pub const BOOL_SERIALIZED_LENGTH: usize = 1;
/// The number of bytes in a serialized `i32`.
pub const I32_SERIALIZED_LENGTH: usize = mem::size_of::<i32>();
/// The number of bytes in a serialized `i64`.
pub const I64_SERIALIZED_LENGTH: usize = mem::size_of::<i64>();
/// The number of bytes in a serialized `u8`.
pub const U8_SERIALIZED_LENGTH: usize = mem::size_of::<u8>();
/// The number of bytes in a serialized `u16`.
pub const U16_SERIALIZED_LENGTH: usize = mem::size_of::<u16>();
/// The number of bytes in a serialized `u32`.
pub const U32_SERIALIZED_LENGTH: usize = mem::size_of::<u32>();
/// The number of bytes in a serialized `u64`.
pub const U64_SERIALIZED_LENGTH: usize = mem::size_of::<u64>();
/// The number of bytes in a serialized [`U128`](crate::U128).
pub const U128_SERIALIZED_LENGTH: usize = mem::size_of::<u128>();
/// The number of bytes in a serialized [`U256`](crate::U256).
pub const U256_SERIALIZED_LENGTH: usize = U128_SERIALIZED_LENGTH * 2;
/// The number of bytes in a serialized [`U512`](crate::U512).
pub const U512_SERIALIZED_LENGTH: usize = U256_SERIALIZED_LENGTH * 2;
/// The tag representing a `None` value.
pub const OPTION_NONE_TAG: u8 = 0;
/// The tag representing a `Some` value.
pub const OPTION_SOME_TAG: u8 = 1;
/// The tag representing an `Err` value.
pub const RESULT_ERR_TAG: u8 = 0;
/// The tag representing an `Ok` value.
pub const RESULT_OK_TAG: u8 = 1;

/// A type which can be serialized to a `Vec<u8>`.
pub trait ToBytes {
    /// Serializes `&self` to a `Vec<u8>`.
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error>;

    /// Returns the length of the `Vec<u8>` which would be returned from a successful call to
    /// `to_bytes()` or `into_bytes()`.  The data is not actually serialized, so this call is
    /// relatively cheap.
    fn serialized_length(&self) -> usize;
}

/// A type which can be deserialized from a `Vec<u8>`.
pub trait FromBytes: Sized {
    /// Deserializes the slice into `Self`.
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error>;

    /// Deserializes the `Vec<u8>` into `Self`.
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}

/// Serialization and deserialization errors.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "std", derive(Error))]
#[repr(u8)]
pub enum Error {
    /// Early end of stream while deserializing.
    #[cfg_attr(feature = "std", error("Deserialization error: early end of stream"))]
    EarlyEndOfStream = 0,
    /// Formatting error while deserializing.
    #[cfg_attr(feature = "std", error("Deserialization error: formatting"))]
    Formatting,
    /// Not all input bytes were consumed in [`deserialize`].
    #[cfg_attr(feature = "std", error("Deserialization error: left-over bytes"))]
    LeftOverBytes,
    /// Out of memory error.
    #[cfg_attr(feature = "std", error("Serialization error: out of memory"))]
    OutOfMemory,
}

/// Serializes `t` into a `Vec<u8>`.
pub fn serialize(t: &impl ToBytes) -> Result<Vec<u8>, Error> {
    let serialized_length = t.serialized_length();
    let mut sink = Vec::with_capacity(serialized_length);
    t.to_bytes(&mut sink)?;
    Ok(sink)
}

/// Deserializes `bytes` into an instance of `T`.
///
/// Returns an error if the bytes cannot be deserialized into `T` or if not all of the input bytes
/// are consumed in the operation.
pub fn deserialize<T: FromBytes>(bytes: Vec<u8>) -> Result<T, Error> {
    let (t, remainder) = T::from_vec(bytes)?;
    if remainder.is_empty() {
        Ok(t)
    } else {
        Err(Error::LeftOverBytes)
    }
}

pub(crate) fn safe_split_at(bytes: &[u8], n: usize) -> Result<(&[u8], &[u8]), Error> {
    if n > bytes.len() {
        Err(Error::EarlyEndOfStream)
    } else {
        Ok(bytes.split_at(n))
    }
}

impl ToBytes for () {
    #[inline(always)]
    fn to_bytes(&self, _sink: &mut Vec<u8>) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        UNIT_SERIALIZED_LENGTH
    }
}

impl FromBytes for () {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        Ok(((), bytes))
    }
}

impl ToBytes for bool {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        u8::from(*self).to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        BOOL_SERIALIZED_LENGTH
    }
}

impl FromBytes for bool {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        match bytes.split_first() {
            None => Err(Error::EarlyEndOfStream),
            Some((byte, rem)) => match byte {
                1 => Ok((true, rem)),
                0 => Ok((false, rem)),
                _ => Err(Error::Formatting),
            },
        }
    }
}

impl ToBytes for u8 {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        sink.push(*self);
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
    }
}

impl FromBytes for u8 {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        match bytes.split_first() {
            None => Err(Error::EarlyEndOfStream),
            Some((byte, rem)) => Ok((*byte, rem)),
        }
    }
}

impl ToBytes for i32 {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        sink.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        I32_SERIALIZED_LENGTH
    }
}

impl FromBytes for i32 {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let mut result = [0u8; I32_SERIALIZED_LENGTH];
        let (bytes, remainder) = safe_split_at(bytes, I32_SERIALIZED_LENGTH)?;
        result.copy_from_slice(bytes);
        Ok((<i32>::from_le_bytes(result), remainder))
    }
}

impl ToBytes for i64 {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        sink.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        I64_SERIALIZED_LENGTH
    }
}

impl FromBytes for i64 {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let mut result = [0u8; I64_SERIALIZED_LENGTH];
        let (bytes, remainder) = safe_split_at(bytes, I64_SERIALIZED_LENGTH)?;
        result.copy_from_slice(bytes);
        Ok((<i64>::from_le_bytes(result), remainder))
    }
}

impl ToBytes for u16 {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        sink.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U16_SERIALIZED_LENGTH
    }
}

impl FromBytes for u16 {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let mut result = [0u8; U16_SERIALIZED_LENGTH];
        let (bytes, remainder) = safe_split_at(bytes, U16_SERIALIZED_LENGTH)?;
        result.copy_from_slice(bytes);
        Ok((<u16>::from_le_bytes(result), remainder))
    }
}

impl ToBytes for u32 {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        sink.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH
    }
}

impl FromBytes for u32 {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let mut result = [0u8; U32_SERIALIZED_LENGTH];
        let (bytes, remainder) = safe_split_at(bytes, U32_SERIALIZED_LENGTH)?;
        result.copy_from_slice(bytes);
        Ok((<u32>::from_le_bytes(result), remainder))
    }
}

impl ToBytes for u64 {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        sink.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U64_SERIALIZED_LENGTH
    }
}

impl FromBytes for u64 {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let mut result = [0u8; U64_SERIALIZED_LENGTH];
        let (bytes, remainder) = safe_split_at(bytes, U64_SERIALIZED_LENGTH)?;
        result.copy_from_slice(bytes);
        Ok((<u64>::from_le_bytes(result), remainder))
    }
}

impl ToBytes for &[u8] {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        let length_prefix = self.len() as u32;
        length_prefix.to_bytes(sink)?;
        sink.extend_from_slice(self);
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH + self.len()
    }
}

impl ToBytes for str {
    #[inline]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.as_bytes().to_bytes(sink)
    }

    #[inline]
    fn serialized_length(&self) -> usize {
        self.as_bytes().serialized_length()
    }
}

impl ToBytes for &str {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        (*self).to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        (*self).serialized_length()
    }
}

impl ToBytes for String {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.as_bytes().to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.as_bytes().serialized_length()
    }
}

impl FromBytes for String {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (size, remainder) = u32::from_bytes(bytes)?;
        let (str_bytes, remainder) = safe_split_at(remainder, size as usize)?;
        let result = String::from_utf8(str_bytes.to_vec()).map_err(|_| Error::Formatting)?;
        Ok((result, remainder))
    }
}

fn ensure_efficient_serialization<T>() {
    #[cfg(debug_assertions)]
    debug_assert_ne!(
        any::type_name::<T>(),
        any::type_name::<u8>(),
        "You should use Bytes newtype wrapper for efficiency"
    );
}

fn iterator_serialized_length<'a, T: 'a + ToBytes>(ts: impl Iterator<Item = &'a T>) -> usize {
    U32_SERIALIZED_LENGTH + ts.map(ToBytes::serialized_length).sum::<usize>()
}

// TODO Replace `try_vec_with_capacity` with `Vec::try_reserve_exact` once it's in stable.
fn try_vec_with_capacity<T>(capacity: usize) -> Result<Vec<T>, Error> {
    // see https://doc.rust-lang.org/src/alloc/raw_vec.rs.html#75-98
    let elem_size = mem::size_of::<T>();
    let alloc_size = capacity.checked_mul(elem_size).ok_or(Error::OutOfMemory)?;

    let ptr = if alloc_size == 0 {
        NonNull::<T>::dangling()
    } else if alloc_size > u32::max_value() as usize {
        return Err(Error::OutOfMemory);
    } else {
        let align = mem::align_of::<T>();
        let layout = Layout::from_size_align(alloc_size, align).map_err(|_| Error::OutOfMemory)?;
        let raw_ptr = unsafe { alloc(layout) };
        let non_null_ptr = NonNull::<u8>::new(raw_ptr).ok_or(Error::OutOfMemory)?;
        non_null_ptr.cast()
    };
    unsafe { Ok(Vec::from_raw_parts(ptr.as_ptr(), 0, capacity)) }
}

fn vec_from_vec<T: FromBytes>(bytes: Vec<u8>) -> Result<(Vec<T>, Vec<u8>), Error> {
    ensure_efficient_serialization::<T>();

    Vec::<T>::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
}

impl<T: ToBytes> ToBytes for Vec<T> {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        ensure_efficient_serialization::<T>();

        let length_prefix = self.len() as u32;
        length_prefix.to_bytes(sink)?;

        for item in self.iter() {
            item.to_bytes(sink)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        iterator_serialized_length(self.iter())
    }
}

impl<T: FromBytes> FromBytes for Vec<T> {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        ensure_efficient_serialization::<T>();

        let (count, mut stream) = u32::from_bytes(bytes)?;

        let mut result = try_vec_with_capacity(count as usize)?;
        for _ in 0..count {
            let (value, remainder) = T::from_bytes(stream)?;
            result.push(value);
            stream = remainder;
        }

        Ok((result, stream))
    }

    #[inline(always)]
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Error> {
        vec_from_vec(bytes)
    }
}

impl<T: ToBytes> ToBytes for VecDeque<T> {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        let length_prefix = self.len() as u32;
        length_prefix.to_bytes(sink)?;

        for item in self.iter() {
            item.to_bytes(sink)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        iterator_serialized_length(self.iter())
    }
}

impl<T: FromBytes> FromBytes for VecDeque<T> {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (vec, bytes) = Vec::from_bytes(bytes)?;
        Ok((VecDeque::from(vec), bytes))
    }

    #[inline(always)]
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Error> {
        let (vec, bytes) = vec_from_vec(bytes)?;
        Ok((VecDeque::from(vec), bytes))
    }
}

macro_rules! impl_to_from_bytes_for_array {
    ($($N:literal)+) => {
        $(
            impl ToBytes for [u8; $N] {
                #[inline(always)]
                fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
                    sink.extend_from_slice(self);
                    Ok(())
                }

                #[inline(always)]
                fn serialized_length(&self) -> usize { $N }
            }

            impl FromBytes for [u8; $N] {
                #[inline(always)]
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
                    let (bytes, rem) = safe_split_at(bytes, $N)?;
                    // SAFETY: safe_split_at makes sure `bytes` is exactly $N bytes.
                    let ptr = bytes.as_ptr() as *const [u8; $N];
                    let result = unsafe { *ptr };
                    Ok((result, rem))
                }
            }
        )+
    }
}

impl_to_from_bytes_for_array! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
    33
    64 128 256 512
}

impl<V: ToBytes> ToBytes for BTreeSet<V> {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        let length_prefix = self.len() as u32;
        length_prefix.to_bytes(sink)?;

        for item in self.iter() {
            item.to_bytes(sink)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        iterator_serialized_length(self.iter())
    }
}

impl<V: FromBytes + Ord> FromBytes for BTreeSet<V> {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (num_keys, mut stream) = u32::from_bytes(bytes)?;
        let mut result = BTreeSet::new();
        for _ in 0..num_keys {
            let (v, rem) = V::from_bytes(stream)?;
            result.insert(v);
            stream = rem;
        }
        Ok((result, stream))
    }
}

impl<K: ToBytes, V: ToBytes> ToBytes for BTreeMap<K, V> {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        let length_prefix = self.len() as u32;
        length_prefix.to_bytes(sink)?;

        for (key, value) in self.iter() {
            key.to_bytes(sink)?;
            value.to_bytes(sink)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH
            + self
                .iter()
                .map(|(key, value)| key.serialized_length() + value.serialized_length())
                .sum::<usize>()
    }
}

impl<K, V> FromBytes for BTreeMap<K, V>
where
    K: FromBytes + Ord,
    V: FromBytes,
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (num_keys, mut stream) = u32::from_bytes(bytes)?;
        let mut result = BTreeMap::new();
        for _ in 0..num_keys {
            let (k, rem) = K::from_bytes(stream)?;
            let (v, rem) = V::from_bytes(rem)?;
            result.insert(k, v);
            stream = rem;
        }
        Ok((result, stream))
    }
}

impl<T: ToBytes> ToBytes for Option<T> {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            None => sink.push(OPTION_NONE_TAG),
            Some(v) => {
                sink.push(OPTION_SOME_TAG);
                v.to_bytes(sink)?;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
            + match self {
                Some(v) => v.serialized_length(),
                None => 0,
            }
    }
}

impl<T: FromBytes> FromBytes for Option<T> {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (tag, rem) = u8::from_bytes(bytes)?;
        match tag {
            OPTION_NONE_TAG => Ok((None, rem)),
            OPTION_SOME_TAG => {
                let (t, rem) = T::from_bytes(rem)?;
                Ok((Some(t), rem))
            }
            _ => Err(Error::Formatting),
        }
    }
}

impl<T: ToBytes, E: ToBytes> ToBytes for Result<T, E> {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            Err(error) => {
                sink.push(RESULT_ERR_TAG);
                error.to_bytes(sink)?;
            }
            Ok(ok) => {
                sink.push(RESULT_OK_TAG);
                ok.to_bytes(sink)?;
            }
        };
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
            + match self {
                Err(error) => error.serialized_length(),
                Ok(ok) => ok.serialized_length(),
            }
    }
}

impl<T: FromBytes, E: FromBytes> FromBytes for Result<T, E> {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (variant, rem) = u8::from_bytes(bytes)?;
        match variant {
            RESULT_ERR_TAG => {
                let (value, rem) = E::from_bytes(rem)?;
                Ok((Err(value), rem))
            }
            RESULT_OK_TAG => {
                let (value, rem) = T::from_bytes(rem)?;
                Ok((Ok(value), rem))
            }
            _ => Err(Error::Formatting),
        }
    }
}

impl<T1: ToBytes> ToBytes for (T1,) {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl<T1: FromBytes> FromBytes for (T1,) {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        Ok(((t1,), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes> ToBytes for (T1, T2) {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length() + self.1.serialized_length()
    }
}

impl<T1: FromBytes, T2: FromBytes> FromBytes for (T1, T2) {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        Ok(((t1, t2), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes, T3: ToBytes> ToBytes for (T1, T2, T3) {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length() + self.1.serialized_length() + self.2.serialized_length()
    }
}

impl<T1: FromBytes, T2: FromBytes, T3: FromBytes> FromBytes for (T1, T2, T3) {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        Ok(((t1, t2, t3), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes, T3: ToBytes, T4: ToBytes> ToBytes for (T1, T2, T3, T4) {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)?;
        self.3.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
            + self.1.serialized_length()
            + self.2.serialized_length()
            + self.3.serialized_length()
    }
}

impl<T1: FromBytes, T2: FromBytes, T3: FromBytes, T4: FromBytes> FromBytes for (T1, T2, T3, T4) {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        let (t4, remainder) = T4::from_bytes(remainder)?;
        Ok(((t1, t2, t3, t4), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes, T3: ToBytes, T4: ToBytes, T5: ToBytes> ToBytes
    for (T1, T2, T3, T4, T5)
{
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)?;
        self.3.to_bytes(sink)?;
        self.4.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
            + self.1.serialized_length()
            + self.2.serialized_length()
            + self.3.serialized_length()
            + self.4.serialized_length()
    }
}

impl<T1: FromBytes, T2: FromBytes, T3: FromBytes, T4: FromBytes, T5: FromBytes> FromBytes
    for (T1, T2, T3, T4, T5)
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        let (t4, remainder) = T4::from_bytes(remainder)?;
        let (t5, remainder) = T5::from_bytes(remainder)?;
        Ok(((t1, t2, t3, t4, t5), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes, T3: ToBytes, T4: ToBytes, T5: ToBytes, T6: ToBytes> ToBytes
    for (T1, T2, T3, T4, T5, T6)
{
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)?;
        self.3.to_bytes(sink)?;
        self.4.to_bytes(sink)?;
        self.5.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
            + self.1.serialized_length()
            + self.2.serialized_length()
            + self.3.serialized_length()
            + self.4.serialized_length()
            + self.5.serialized_length()
    }
}

impl<T1: FromBytes, T2: FromBytes, T3: FromBytes, T4: FromBytes, T5: FromBytes, T6: FromBytes>
    FromBytes for (T1, T2, T3, T4, T5, T6)
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        let (t4, remainder) = T4::from_bytes(remainder)?;
        let (t5, remainder) = T5::from_bytes(remainder)?;
        let (t6, remainder) = T6::from_bytes(remainder)?;
        Ok(((t1, t2, t3, t4, t5, t6), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes, T3: ToBytes, T4: ToBytes, T5: ToBytes, T6: ToBytes, T7: ToBytes>
    ToBytes for (T1, T2, T3, T4, T5, T6, T7)
{
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)?;
        self.3.to_bytes(sink)?;
        self.4.to_bytes(sink)?;
        self.5.to_bytes(sink)?;
        self.6.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
            + self.1.serialized_length()
            + self.2.serialized_length()
            + self.3.serialized_length()
            + self.4.serialized_length()
            + self.5.serialized_length()
            + self.6.serialized_length()
    }
}

impl<
        T1: FromBytes,
        T2: FromBytes,
        T3: FromBytes,
        T4: FromBytes,
        T5: FromBytes,
        T6: FromBytes,
        T7: FromBytes,
    > FromBytes for (T1, T2, T3, T4, T5, T6, T7)
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        let (t4, remainder) = T4::from_bytes(remainder)?;
        let (t5, remainder) = T5::from_bytes(remainder)?;
        let (t6, remainder) = T6::from_bytes(remainder)?;
        let (t7, remainder) = T7::from_bytes(remainder)?;
        Ok(((t1, t2, t3, t4, t5, t6, t7), remainder))
    }
}

impl<
        T1: ToBytes,
        T2: ToBytes,
        T3: ToBytes,
        T4: ToBytes,
        T5: ToBytes,
        T6: ToBytes,
        T7: ToBytes,
        T8: ToBytes,
    > ToBytes for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)?;
        self.3.to_bytes(sink)?;
        self.4.to_bytes(sink)?;
        self.5.to_bytes(sink)?;
        self.6.to_bytes(sink)?;
        self.7.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
            + self.1.serialized_length()
            + self.2.serialized_length()
            + self.3.serialized_length()
            + self.4.serialized_length()
            + self.5.serialized_length()
            + self.6.serialized_length()
            + self.7.serialized_length()
    }
}

impl<
        T1: FromBytes,
        T2: FromBytes,
        T3: FromBytes,
        T4: FromBytes,
        T5: FromBytes,
        T6: FromBytes,
        T7: FromBytes,
        T8: FromBytes,
    > FromBytes for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        let (t4, remainder) = T4::from_bytes(remainder)?;
        let (t5, remainder) = T5::from_bytes(remainder)?;
        let (t6, remainder) = T6::from_bytes(remainder)?;
        let (t7, remainder) = T7::from_bytes(remainder)?;
        let (t8, remainder) = T8::from_bytes(remainder)?;
        Ok(((t1, t2, t3, t4, t5, t6, t7, t8), remainder))
    }
}

impl<
        T1: ToBytes,
        T2: ToBytes,
        T3: ToBytes,
        T4: ToBytes,
        T5: ToBytes,
        T6: ToBytes,
        T7: ToBytes,
        T8: ToBytes,
        T9: ToBytes,
    > ToBytes for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
{
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)?;
        self.3.to_bytes(sink)?;
        self.4.to_bytes(sink)?;
        self.5.to_bytes(sink)?;
        self.6.to_bytes(sink)?;
        self.7.to_bytes(sink)?;
        self.8.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
            + self.1.serialized_length()
            + self.2.serialized_length()
            + self.3.serialized_length()
            + self.4.serialized_length()
            + self.5.serialized_length()
            + self.6.serialized_length()
            + self.7.serialized_length()
            + self.8.serialized_length()
    }
}

impl<
        T1: FromBytes,
        T2: FromBytes,
        T3: FromBytes,
        T4: FromBytes,
        T5: FromBytes,
        T6: FromBytes,
        T7: FromBytes,
        T8: FromBytes,
        T9: FromBytes,
    > FromBytes for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        let (t4, remainder) = T4::from_bytes(remainder)?;
        let (t5, remainder) = T5::from_bytes(remainder)?;
        let (t6, remainder) = T6::from_bytes(remainder)?;
        let (t7, remainder) = T7::from_bytes(remainder)?;
        let (t8, remainder) = T8::from_bytes(remainder)?;
        let (t9, remainder) = T9::from_bytes(remainder)?;
        Ok(((t1, t2, t3, t4, t5, t6, t7, t8, t9), remainder))
    }
}

impl<
        T1: ToBytes,
        T2: ToBytes,
        T3: ToBytes,
        T4: ToBytes,
        T5: ToBytes,
        T6: ToBytes,
        T7: ToBytes,
        T8: ToBytes,
        T9: ToBytes,
        T10: ToBytes,
    > ToBytes for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
{
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        self.0.to_bytes(sink)?;
        self.1.to_bytes(sink)?;
        self.2.to_bytes(sink)?;
        self.3.to_bytes(sink)?;
        self.4.to_bytes(sink)?;
        self.5.to_bytes(sink)?;
        self.6.to_bytes(sink)?;
        self.7.to_bytes(sink)?;
        self.8.to_bytes(sink)?;
        self.9.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
            + self.1.serialized_length()
            + self.2.serialized_length()
            + self.3.serialized_length()
            + self.4.serialized_length()
            + self.5.serialized_length()
            + self.6.serialized_length()
            + self.7.serialized_length()
            + self.8.serialized_length()
            + self.9.serialized_length()
    }
}

impl<
        T1: FromBytes,
        T2: FromBytes,
        T3: FromBytes,
        T4: FromBytes,
        T5: FromBytes,
        T6: FromBytes,
        T7: FromBytes,
        T8: FromBytes,
        T9: FromBytes,
        T10: FromBytes,
    > FromBytes for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        let (t4, remainder) = T4::from_bytes(remainder)?;
        let (t5, remainder) = T5::from_bytes(remainder)?;
        let (t6, remainder) = T6::from_bytes(remainder)?;
        let (t7, remainder) = T7::from_bytes(remainder)?;
        let (t8, remainder) = T8::from_bytes(remainder)?;
        let (t9, remainder) = T9::from_bytes(remainder)?;
        let (t10, remainder) = T10::from_bytes(remainder)?;
        Ok(((t1, t2, t3, t4, t5, t6, t7, t8, t9, t10), remainder))
    }
}

impl<T> ToBytes for Ratio<T>
where
    T: Clone + Integer + ToBytes,
{
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        if self.denom().is_zero() {
            return Err(Error::Formatting);
        }
        (self.numer().clone(), self.denom().clone()).to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        (self.numer().clone(), self.denom().clone()).serialized_length()
    }
}

impl<T> FromBytes for Ratio<T>
where
    T: Clone + FromBytes + Integer,
{
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let ((numer, denom), rem): ((T, T), &[u8]) = FromBytes::from_bytes(bytes)?;
        if denom.is_zero() {
            return Err(Error::Formatting);
        }
        Ok((Ratio::new(numer, denom), rem))
    }
}

// This test helper is not intended to be used by third party crates.
#[doc(hidden)]
/// Returns `true` if a we can serialize and then deserialize a value
pub fn test_serialization_roundtrip<T>(t: &T)
where
    T: alloc::fmt::Debug + ToBytes + FromBytes + PartialEq,
{
    let serialized = serialize(t).expect("Unable to serialize data");
    assert_eq!(
        serialized.len(),
        t.serialized_length(),
        "\nLength of serialized data: {},\nserialized_length() yielded: {},\nserialized data: {:?}, t is {:?}",
        serialized.len(),
        t.serialized_length(),
        serialized,
        t
    );
    let deserialized = deserialize::<T>(serialized).expect("Unable to deserialize data");
    assert!(*t == deserialized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_not_serialize_zero_denominator() {
        let malicious = Ratio::new_raw(1, 0);
        assert_eq!(serialize(&malicious).unwrap_err(), Error::Formatting);
    }

    #[test]
    fn should_not_deserialize_zero_denominator() {
        let malicious_bytes = serialize(&(1u64, 0u64)).unwrap();
        let result: Result<Ratio<u64>, Error> = super::deserialize(malicious_bytes);
        assert_eq!(result.unwrap_err(), Error::Formatting);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "You should use Bytes newtype wrapper for efficiency")]
    fn should_fail_to_serialize_slice_of_u8() {
        let bytes = b"0123456789".to_vec();
        serialize(&bytes).unwrap();
    }
}

#[cfg(test)]
mod proptests {
    use std::collections::VecDeque;

    use proptest::{collection::vec, prelude::*};

    use crate::{
        bytesrepr::{self, bytes::gens::bytes_arb},
        gens::*,
    };

    proptest! {
        #[test]
        fn test_bool(u in any::<bool>()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u8(u in any::<u8>()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u16(u in any::<u16>()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u32(u in any::<u32>()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_i32(u in any::<i32>()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u64(u in any::<u64>()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_i64(u in any::<i64>()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u8_slice_32(s in u8_slice_32()) {
            bytesrepr::test_serialization_roundtrip(&s);
        }

        #[test]
        fn test_vec_u8(u in bytes_arb(1..100)) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_vec_i32(u in vec(any::<i32>(), 1..100)) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_vecdeque_i32((front, back) in (vec(any::<i32>(), 1..100), vec(any::<i32>(), 1..100))) {
            let mut vec_deque = VecDeque::new();
            for f in front {
                vec_deque.push_front(f);
            }
            for f in back {
                vec_deque.push_back(f);
            }
            bytesrepr::test_serialization_roundtrip(&vec_deque);
        }

        #[test]
        fn test_vec_vec_u8(u in vec(bytes_arb(1..100), 10)) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_uref_map(m in named_keys_arb(20)) {
            bytesrepr::test_serialization_roundtrip(&m);
        }

        #[test]
        fn test_array_u8_32(arr in any::<[u8; 32]>()) {
            bytesrepr::test_serialization_roundtrip(&arr);
        }

        #[test]
        fn test_string(s in "\\PC*") {
            bytesrepr::test_serialization_roundtrip(&s);
        }

        #[test]
        fn test_str(s in "\\PC*") {
            let not_a_string_object = s.as_str();
            bytesrepr::serialize(&not_a_string_object).expect("should serialize a str");
        }

        #[test]
        fn test_option(o in proptest::option::of(key_arb())) {
            bytesrepr::test_serialization_roundtrip(&o);
        }

        #[test]
        fn test_unit(unit in Just(())) {
            bytesrepr::test_serialization_roundtrip(&unit);
        }

        #[test]
        fn test_u128_serialization(u in u128_arb()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u256_serialization(u in u256_arb()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u512_serialization(u in u512_arb()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_key_serialization(key in key_arb()) {
            bytesrepr::test_serialization_roundtrip(&key);
        }

        #[test]
        fn test_cl_value_serialization(cl_value in cl_value_arb()) {
            bytesrepr::test_serialization_roundtrip(&cl_value);
        }

        #[test]
        fn test_access_rights(access_right in access_rights_arb()) {
            bytesrepr::test_serialization_roundtrip(&access_right);
        }

        #[test]
        fn test_uref(uref in uref_arb()) {
            bytesrepr::test_serialization_roundtrip(&uref);
        }

        #[test]
        fn test_account_hash(pk in account_hash_arb()) {
            bytesrepr::test_serialization_roundtrip(&pk);
        }

        #[test]
        fn test_result(result in result_arb()) {
            bytesrepr::test_serialization_roundtrip(&result);
        }

        #[test]
        fn test_phase_serialization(phase in phase_arb()) {
            bytesrepr::test_serialization_roundtrip(&phase);
        }

        #[test]
        fn test_protocol_version(protocol_version in protocol_version_arb()) {
            bytesrepr::test_serialization_roundtrip(&protocol_version);
        }

        #[test]
        fn test_sem_ver(sem_ver in sem_ver_arb()) {
            bytesrepr::test_serialization_roundtrip(&sem_ver);
        }

        #[test]
        fn test_tuple1(t in (any::<u8>(),)) {
            bytesrepr::test_serialization_roundtrip(&t);
        }

        #[test]
        fn test_tuple2(t in (any::<u8>(),any::<u32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }

        #[test]
        fn test_tuple3(t in (any::<u8>(),any::<u32>(),any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }

        #[test]
        fn test_tuple4(t in (any::<u8>(),any::<u32>(),any::<i32>(), any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
        #[test]
        fn test_tuple5(t in (any::<u8>(),any::<u32>(),any::<i32>(), any::<i32>(), any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
        #[test]
        fn test_tuple6(t in (any::<u8>(),any::<u32>(),any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
        #[test]
        fn test_tuple7(t in (any::<u8>(),any::<u32>(),any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
        #[test]
        fn test_tuple8(t in (any::<u8>(),any::<u32>(),any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
        #[test]
        fn test_tuple9(t in (any::<u8>(),any::<u32>(),any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
        #[test]
        fn test_tuple10(t in (any::<u8>(),any::<u32>(),any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
        #[test]
        fn test_ratio_u64(t in (any::<u64>(), 1..u64::max_value())) {
            bytesrepr::test_serialization_roundtrip(&t);
        }
    }
}
