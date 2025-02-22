// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::Version;
use crate::{proof::SparseMerkleRangeProof, state_store::state_key::StateKey};
use aptos_crypto::{
    hash::{CryptoHash, CryptoHasher, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use aptos_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{arbitrary::Arbitrary, prelude::*};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Default, CryptoHasher, Eq, PartialEq, Serialize, Ord, PartialOrd, Hash)]
pub struct StateValue {
    pub bytes: Vec<u8>,
    #[serde(skip)]
    hash: HashValue,
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for StateValue {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        any::<Vec<u8>>().prop_map(StateValue::new).boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

impl<'de> Deserialize<'de> for StateValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "StateValue")]
        struct Bytes {
            bytes: Vec<u8>,
        }
        let bytes = Bytes::deserialize(deserializer)?;

        Ok(Self::new(bytes.bytes))
    }
}

impl StateValue {
    fn new(bytes: Vec<u8>) -> Self {
        let mut hasher = StateValueHasher::default();
        hasher.update(bytes.as_slice());
        let hash = hasher.finish();
        Self { bytes, hash }
    }

    pub fn size(&self) -> usize {
        self.bytes.len()
    }
}

impl From<Vec<u8>> for StateValue {
    fn from(bytes: Vec<u8>) -> Self {
        StateValue::new(bytes)
    }
}

impl CryptoHash for StateValue {
    type Hasher = StateValueHasher;

    fn hash(&self) -> HashValue {
        self.hash
    }
}

/// TODO(joshlind): add a proof implementation (e.g., verify()) and unit tests
/// for these once we start supporting them.
///
/// A single chunk of all state values at a specific version.
/// Note: this is similar to `StateSnapshotChunk` but all data is included
/// in the struct itself and not behind pointers/handles to file locations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
pub struct StateValueChunkWithProof {
    pub first_index: u64,     // The first hashed state index in chunk
    pub last_index: u64,      // The last hashed state index in chunk
    pub first_key: HashValue, // The first hashed state key in chunk
    pub last_key: HashValue,  // The last hashed state key in chunk
    pub raw_values: Vec<(StateKey, StateValue)>, // The hashed state key and and raw state value.
    pub proof: SparseMerkleRangeProof, // The proof to ensure the chunk is in the hashed states
    pub root_hash: HashValue, // The root hash of the sparse merkle tree for this chunk
}

impl StateValueChunkWithProof {
    /// Returns true iff this chunk is the last chunk (i.e., there are no
    /// more state values to write to storage after this chunk).
    pub fn is_last_chunk(&self) -> bool {
        let right_siblings = self.proof.right_siblings();
        right_siblings
            .iter()
            .all(|sibling| *sibling == *SPARSE_MERKLE_PLACEHOLDER_HASH)
    }
}

/// Indicates a state value becomes stale since `stale_since_version`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
pub struct StaleStateValueIndex {
    /// The version since when the node is overwritten and becomes stale.
    pub stale_since_version: Version,
    /// The version identifying the value associated with this record.
    pub version: Version,
    /// The `StateKey` identifying the value associated with this record.
    pub state_key: StateKey,
}
