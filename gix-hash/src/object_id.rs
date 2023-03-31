use crate::{borrowed::oid, Kind, SIZE_OF_SHA1_DIGEST};
use std::{
    borrow::Borrow,
    convert::TryInto,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};
#[doc = " An owned hash identifying objects, most commonly Sha1"]
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectId {
    #[doc = " A SHA 1 hash digest"]
    Sha1([u8; SIZE_OF_SHA1_DIGEST]),
}
#[allow(clippy::derive_hash_xor_eq)]
impl Hash for ObjectId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_slice())
    }
}
#[allow(missing_docs)]
pub mod decode {
    use crate::object_id::ObjectId;
    use hex::FromHex;
    use std::str::FromStr;
    #[doc = " An error returned by [`ObjectId::from_hex()`][crate::ObjectId::from_hex()]"]
    #[derive(Debug, thiserror :: Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error("A hash sized {0} hexadecimal characters is invalid")]
        InvalidHexEncodingLength(usize),
        #[error("Invalid character {c} at position {index}")]
        Invalid { c: char, index: usize },
    }
    #[doc = " Hash decoding"]
    impl ObjectId {
        #[doc = " Create an instance from a `buffer` of 40 bytes encoded with hexadecimal notation."]
        #[doc = ""]
        #[doc = " Such a buffer can be obtained using [`oid::write_hex_to(buffer)`][super::oid::write_hex_to()]"]
        pub fn from_hex(buffer: &[u8]) -> Result<ObjectId, Error> {
            match buffer.len() {
                40 => Self::bar(buffer),
                len => Err(Error::InvalidHexEncodingLength(len)),
            }
        }
        fn bar(buffer: &[u8]) -> Result<ObjectId, Error> {
            Ok(ObjectId::Sha1(<[u8; 20]>::from_hex(buffer).map_err(
                |err| match err {
                    hex::FromHexError::InvalidHexCharacter { c, index } => {
                        Error::Invalid { c, index }
                    }
                    hex::FromHexError::OddLength | hex::FromHexError::InvalidStringLength => {
                        unreachable!("BUG: This is already checked")
                    }
                },
            )?))
        }
    }
    impl FromStr for ObjectId {
        type Err = Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Self::from_hex(s.as_bytes())
        }
    }
}
#[doc = " Access and conversion"]
impl ObjectId {
    #[doc = " Returns the kind of hash used in this `Id`"]
    #[inline]
    pub fn kind(&self) -> crate::Kind {
        match self {
            ObjectId::Sha1(_) => crate::Kind::Sha1,
        }
    }
    #[doc = " Return the raw byte slice representing this hash"]
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Sha1(b) => b.as_ref(),
        }
    }
    #[doc = " Return the raw mutable byte slice representing this hash"]
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            Self::Sha1(b) => b.as_mut(),
        }
    }
    #[doc = " The hash of an empty blob"]
    #[inline]
    pub const fn empty_blob(hash: Kind) -> ObjectId {
        match hash { Kind :: Sha1 => { ObjectId :: Sha1 (* b"\xe6\x9d\xe2\x9b\xb2\xd1\xd6\x43\x4b\x8b\x29\xae\x77\x5a\xd8\xc2\xe4\x8c\x53\x91") } }
    }
    #[doc = " The hash of an empty tree"]
    #[inline]
    pub const fn empty_tree(hash: Kind) -> ObjectId {
        match hash { Kind :: Sha1 => { ObjectId :: Sha1 (* b"\x4b\x82\x5d\xc6\x42\xcb\x6e\xb9\xa0\x60\xe5\x4b\xf8\xd6\x92\x88\xfb\xee\x49\x04") } }
    }
    #[doc = " Returns true if this hash consists of all null bytes"]
    #[inline]
    pub fn is_null(&self) -> bool {
        match self {
            ObjectId::Sha1(digest) => &digest[..] == oid::null_sha1().as_bytes(),
        }
    }
    #[doc = " Returns an Digest representing a hash with whose memory is zeroed."]
    #[inline]
    pub const fn null(kind: crate::Kind) -> ObjectId {
        match kind {
            crate::Kind::Sha1 => Self::null_sha1(),
        }
    }
}
#[doc = " Sha1 hash specific methods"]
impl ObjectId {
    #[doc = " Instantiate an Digest from 20 bytes of a Sha1 digest."]
    #[inline]
    fn new_sha1(id: [u8; SIZE_OF_SHA1_DIGEST]) -> Self {
        ObjectId::Sha1(id)
    }
    #[doc = " Instantiate an Digest from a slice 20 borrowed bytes of a Sha1 digest."]
    #[doc = ""]
    #[doc = " Panics of the slice doesn't have a length of 20."]
    #[inline]
    pub(crate) fn from_20_bytes(b: &[u8]) -> ObjectId {
        let mut id = [0; SIZE_OF_SHA1_DIGEST];
        id.copy_from_slice(b);
        ObjectId::Sha1(id)
    }
    #[doc = " Returns an Digest representing a Sha1 with whose memory is zeroed."]
    #[inline]
    pub(crate) const fn null_sha1() -> ObjectId {
        ObjectId::Sha1([0u8; 20])
    }
}
impl std::fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectId::Sha1(_hash) => f.write_str("Sha1(")?,
        }
        for b in self.as_bytes() {
            write!(f, "{b:02x}")?;
        }
        f.write_str(")")
    }
}
impl From<[u8; SIZE_OF_SHA1_DIGEST]> for ObjectId {
    fn from(v: [u8; 20]) -> Self {
        Self::new_sha1(v)
    }
}
impl From<&[u8]> for ObjectId {
    fn from(v: &[u8]) -> Self {
        match v.len() {
            20 => Self::Sha1(v.try_into().expect("prior length validation")),
            other => panic!("BUG: unsupported hash len: {other}"),
        }
    }
}
impl From<&crate::oid> for ObjectId {
    fn from(v: &oid) -> Self {
        match v.kind() {
            crate::Kind::Sha1 => ObjectId::from_20_bytes(v.as_bytes()),
        }
    }
}
impl Deref for ObjectId {
    type Target = oid;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl AsRef<crate::oid> for ObjectId {
    fn as_ref(&self) -> &oid {
        oid::from_bytes_unchecked(self.as_slice())
    }
}
impl Borrow<crate::oid> for ObjectId {
    fn borrow(&self) -> &oid {
        self.as_ref()
    }
}
impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}
impl PartialEq<&crate::oid> for ObjectId {
    fn eq(&self, other: &&oid) -> bool {
        self.as_ref() == *other
    }
}
