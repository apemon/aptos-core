// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

//! This module provides common utilities for the DB pruner.

use crate::{
    pruner::{ledger_store::ledger_store_pruner::LedgerPruner, state_store::StateMerklePruner},
    EventStore, LedgerStore, TransactionStore,
};

use schemadb::DB;
use std::sync::Arc;

/// A utility function to instantiate the state pruner
pub fn create_state_pruner(state_merkle_db: Arc<DB>) -> Arc<StateMerklePruner> {
    Arc::new(StateMerklePruner::new(Arc::clone(&state_merkle_db)))
}

/// A utility function to instantiate the ledger pruner
pub fn create_ledger_pruner(ledger_db: Arc<DB>) -> Arc<LedgerPruner> {
    Arc::new(LedgerPruner::new(
        Arc::clone(&ledger_db),
        Arc::new(TransactionStore::new(Arc::clone(&ledger_db))),
        Arc::new(EventStore::new(Arc::clone(&ledger_db))),
        Arc::new(LedgerStore::new(Arc::clone(&ledger_db))),
    ))
}
