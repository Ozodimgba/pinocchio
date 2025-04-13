#![no_std]
#![cfg_attr(feature = "std", allow(unused_imports))]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use {
    core::fmt,
    core::str::FromStr,
    std::string::{String, ToString},
    base58::{ToBase58, FromBase58},
};

#[cfg(feature = "borsh")]
use borsh::{BorshDeserialize, BorshSerialize};

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

pub const HASH_BYTES: usize = 32;

#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[cfg_attr(feature = "std", derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash))]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Hash(pub [u8; HASH_BYTES]);

#[cfg(feature = "std")]
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_slice().to_base58())
    }
}

#[cfg(feature = "std")]
impl FromStr for Hash {
    type Err = base58::FromBase58Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.from_base58()?;
        if bytes.len() != HASH_BYTES {
            return Err(base58::FromBase58Error::InvalidBase58Character(' ', 0));
        }
        let mut array = [0u8; HASH_BYTES];
        array.copy_from_slice(&bytes);
        Ok(Hash(array))
    }
}

#[cfg(feature = "std")]
impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Hash {
    pub const fn new_from_array(hash_array: [u8; HASH_BYTES]) -> Self {
        Self(hash_array)
    }

    pub const fn to_bytes(self) -> [u8; HASH_BYTES] {
        self.0
    }

    #[cfg(feature = "std")]
    pub fn new_unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut bytes = [0u8; HASH_BYTES];
        bytes[0..8].copy_from_slice(&count.to_le_bytes());
        Hash(bytes)
    }
}

pub fn hash(val: &[u8]) -> Hash {
    hashv(&[val])
}

pub fn hashv(vals: &[&[u8]]) -> Hash {
    let mut hash_result = [0u8; HASH_BYTES];

    let mut total_len: u64 = 0;
    for &val in vals {
        total_len += val.len() as u64;
    }

    let data = alloc_buffer(total_len as usize);
    let mut cursor = 0;
    for &val in vals {
        data[cursor..cursor + val.len()].copy_from_slice(val);
        cursor += val.len();
    }

    unsafe {
        sol_keccak256(
            data.as_ptr(),
            total_len,
            hash_result.as_mut_ptr(),
        );
    }

    Hash::new_from_array(hash_result)
}

#[cfg_attr(test, allow(dead_code))]
extern "C" {
    fn sol_keccak256(vals: *const u8, val_len: u64, hash_result: *mut u8);
}

#[cfg_attr(test, allow(static_mut_refs))]
fn alloc_buffer(size: usize) -> &'static mut [u8] {
    static mut BUFFER: [u8; 1024] = [0; 1024];
    unsafe { core::slice::from_raw_parts_mut(BUFFER.as_mut_ptr(), size) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use std::vec;

    #[test]
    fn test_hash_new() {
        let bytes = [1u8; HASH_BYTES];
        let hash = Hash::new_from_array(bytes);
        assert_eq!(hash.to_bytes(), bytes);
    }

    #[cfg(all(feature = "borsh", feature = "std"))]
    #[test]
    fn test_borsh() {
        use borsh::BorshSerialize;
        let hash = Hash([1u8; 32]);
        let bytes = borsh::to_vec(&hash).unwrap();
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes, vec![1u8; 32]);
        let deserialized = Hash::try_from_slice(&bytes).unwrap();
        assert_eq!(hash.0, deserialized.0);
    }

    #[cfg(feature = "bytemuck")]
    #[test]
    fn test_bytemuck() {
        let hash = Hash([1u8; 32]);
        let bytes: &[u8; 32] = bytemuck::cast_ref(&hash);
        assert_eq!(bytes.len(), 32);
        assert_eq!(*bytes, [1u8; 32]);
        let deserialized = bytemuck::try_cast::<[u8; 32], Hash>(*bytes).unwrap();
        assert_eq!(hash.0, deserialized.0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_std_formatting() {
        let hash = Hash([0u8; 32]);
        let s = hash.to_string();
        assert_eq!(s, "11111111111111111111111111111111");
        let parsed = Hash::from_str(&s).unwrap();
        assert_eq!(parsed.0, [0u8; 32]);
        assert_eq!(hash.to_string(), s);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_new_unique() {
        let hash1 = Hash::new_unique();
        let hash2 = Hash::new_unique();
        assert_ne!(hash1.0, hash2.0);
    }
}