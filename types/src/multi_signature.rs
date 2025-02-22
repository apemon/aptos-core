// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use aptos_crypto::bls12381;
use aptos_crypto_derive::{BCSCryptoHash, CryptoHasher};
use std::collections::HashMap;

use aptos_bitvec::BitVec;
use move_deps::move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

/// This struct represents a BLS multi-signature: it stores a bit mask representing the set of
/// validators participating in the signing process and the multi-signature itself, which was
/// aggregated from these validators' partial BLS signatures.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub struct MultiSignature {
    validator_bitmask: BitVec,
    multi_sig: Option<bls12381::Signature>,
}

impl MultiSignature {
    pub fn new(
        validator_bitmask: BitVec,
        aggregated_signature: Option<bls12381::Signature>,
    ) -> Self {
        Self {
            validator_bitmask,
            multi_sig: aggregated_signature,
        }
    }

    pub fn empty() -> Self {
        Self {
            validator_bitmask: BitVec::default(),
            multi_sig: None,
        }
    }

    pub fn get_voters_bitvec(&self) -> &BitVec {
        &self.validator_bitmask
    }

    pub fn get_voter_addresses(
        &self,
        validator_addresses: &[AccountAddress],
    ) -> Vec<AccountAddress> {
        validator_addresses
            .iter()
            .enumerate()
            .filter_map(|(index, addr)| {
                if self.validator_bitmask.is_set(index as u16) {
                    Some(*addr)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_num_voters(&self) -> usize {
        self.validator_bitmask.count_ones() as usize
    }

    pub fn multi_sig(&self) -> &Option<bls12381::Signature> {
        &self.multi_sig
    }
}

/// Partial signature from a set of validators. This struct is only used when aggregating the votes
/// from different validators. It is only kept in memory and never sent through the network.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialSignatures {
    signatures: HashMap<AccountAddress, bls12381::Signature>,
}

impl PartialSignatures {
    pub fn new(signatures: HashMap<AccountAddress, bls12381::Signature>) -> Self {
        Self { signatures }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }

    pub fn remove_signature(&mut self, validator: AccountAddress) {
        self.signatures.remove(&validator);
    }

    pub fn add_signature(&mut self, validator: AccountAddress, signature: bls12381::Signature) {
        self.signatures.entry(validator).or_insert(signature);
    }

    pub fn signatures(&self) -> &HashMap<AccountAddress, bls12381::Signature> {
        &self.signatures
    }
}
