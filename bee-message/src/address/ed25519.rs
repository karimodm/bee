// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{unlock::Ed25519Signature, Error};

use bee_common::packable::{Packable, Read, Write};

use bech32::{self, ToBase32, Variant};
use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519::{PublicKey, Signature},
};

use alloc::{string::String, vec};
use core::{convert::TryInto, str::FromStr};

pub const ED25519_ADDRESS_LENGTH: usize = 32;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ed25519Address([u8; ED25519_ADDRESS_LENGTH]);

impl Ed25519Address {
    pub const KIND: u8 = 0;

    pub fn new(address: [u8; ED25519_ADDRESS_LENGTH]) -> Self {
        address.into()
    }

    pub fn len(&self) -> usize {
        ED25519_ADDRESS_LENGTH
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // TODO we should probably not have this method, only go through the enum's one (because of the type byte).
    pub fn to_bech32(&self, hrp: &str) -> String {
        let mut serialized = vec![Self::KIND];
        serialized.extend_from_slice(&self.0);
        bech32::encode(hrp, serialized.to_base32(), Variant::Bech32).expect("Valid Ed25519 address required.")
    }

    pub fn verify(&self, msg: &[u8], signature: &Ed25519Signature) -> Result<(), Error> {
        let address = Blake2b256::digest(signature.public_key());

        if self.0 != *address {
            return Err(Error::SignaturePublicKeyMismatch(
                hex::encode(self.0),
                hex::encode(address),
            ));
        }

        // TODO unwraps are temporary until we use crypto.rs types as internals.

        if !PublicKey::from_compressed_bytes(*signature.public_key())
            .unwrap()
            .verify(&Signature::from_bytes(signature.signature().try_into().unwrap()), msg)
        {
            return Err(Error::InvalidSignature);
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(Ed25519Address);

impl From<[u8; ED25519_ADDRESS_LENGTH]> for Ed25519Address {
    fn from(bytes: [u8; ED25519_ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for Ed25519Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; ED25519_ADDRESS_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(ED25519_ADDRESS_LENGTH * 2, s.len()))?;

        Ok(Ed25519Address::from(bytes))
    }
}

impl AsRef<[u8]> for Ed25519Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Ed25519Address({})", self)
    }
}

impl Packable for Ed25519Address {
    type Error = Error;

    fn packed_len(&self) -> usize {
        ED25519_ADDRESS_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; ED25519_ADDRESS_LENGTH];
        reader.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
