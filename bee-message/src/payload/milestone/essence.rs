// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    Error, Parents,
};

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable, Read, Write},
};

use crypto::hashes::{blake2b::Blake2b256, Digest};

use alloc::vec::Vec;

pub const MILESTONE_MERKLE_PROOF_LENGTH: usize = 32;
pub const MILESTONE_PUBLIC_KEY_LENGTH: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestonePayloadEssence {
    index: MilestoneIndex,
    timestamp: u64,
    parents: Parents,
    merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
    public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
    receipt: Option<Payload>,
}

impl MilestonePayloadEssence {
    pub fn new(
        index: MilestoneIndex,
        timestamp: u64,
        parents: Parents,
        merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
        public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
        receipt: Option<Payload>,
    ) -> Result<Self, Error> {
        if !matches!(receipt, None | Some(Payload::Receipt(_))) {
            // Safe to unwrap since it's known not to be None.
            return Err(Error::InvalidPayloadKind(receipt.unwrap().kind()));
        }

        if public_keys.is_empty() {
            return Err(Error::MilestoneNoPublicKey);
        }

        if !is_unique_sorted(public_keys.iter()) {
            return Err(Error::MilestonePublicKeysNotUniqueSorted);
        }

        Ok(Self {
            index,
            timestamp,
            parents,
            merkle_proof,
            public_keys,
            receipt,
        })
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }

    pub fn public_keys(&self) -> &Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]> {
        &self.public_keys
    }

    pub fn receipt(&self) -> Option<&Payload> {
        self.receipt.as_ref()
    }

    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_new()).into()
    }
}

impl Packable for MilestonePayloadEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.timestamp.packed_len()
            + self.parents.packed_len()
            + MILESTONE_MERKLE_PROOF_LENGTH
            + 0u8.packed_len()
            + self.public_keys.len() * MILESTONE_PUBLIC_KEY_LENGTH
            + option_payload_packed_len(self.receipt.as_ref())
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;
        self.timestamp.pack(writer)?;
        self.parents.pack(writer)?;
        writer.write_all(&self.merkle_proof)?;
        (self.public_keys.len() as u8).pack(writer)?;
        for public_key in &self.public_keys {
            writer.write_all(public_key)?;
        }
        option_payload_pack(writer, self.receipt.as_ref())?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = MilestoneIndex::unpack(reader)?;
        let timestamp = u64::unpack(reader)?;
        let parents = Parents::unpack(reader)?;

        let mut merkle_proof = [0u8; MILESTONE_MERKLE_PROOF_LENGTH];
        reader.read_exact(&mut merkle_proof)?;

        let public_keys_len = u8::unpack(reader)? as usize;
        let mut public_keys = Vec::with_capacity(public_keys_len);
        for _ in 0..public_keys_len {
            let mut public_key = [0u8; MILESTONE_PUBLIC_KEY_LENGTH];
            reader.read_exact(&mut public_key)?;
            public_keys.push(public_key);
        }

        let (_, receipt) = option_payload_unpack(reader)?;

        // TODO builder ?

        Self::new(index, timestamp, parents, merkle_proof, public_keys, receipt)
    }
}
