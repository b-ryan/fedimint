//! Sled implementation of the `Database` trait. It should not be used anymore since it has known
//! issues and is unmaintained. Please use `rocksdb` instead.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use fedimint_api::db::{
    DatabaseDeleteOperation, DatabaseInsertOperation, DatabaseOperation, DatabaseTransaction,
    PrefixIter,
};
use fedimint_api::db::{IDatabase, IDatabaseTransaction};
pub use sled;
use sled::transaction::TransactionError;

#[derive(Debug)]
pub struct SledDb(sled::Tree);

#[derive(Debug)]
pub struct SledTransaction<'a> {
    operations: Vec<DatabaseOperation>,
    db: &'a SledDb,
    num_pending_operations: usize,
    num_savepoint_operations: usize,
}

impl SledDb {
    pub fn open(db_path: impl AsRef<Path>, tree: &str) -> Result<SledDb, sled::Error> {
        let db = sled::open(db_path)?.open_tree(tree)?;
        Ok(SledDb(db))
    }

    pub fn inner(&self) -> &sled::Tree {
        &self.0
    }
}

impl From<sled::Tree> for SledDb {
    fn from(db: sled::Tree) -> Self {
        SledDb(db)
    }
}

impl From<SledDb> for sled::Tree {
    fn from(db: SledDb) -> Self {
        db.0
    }
}

// TODO: maybe make the concrete impl its own crate
impl IDatabase for SledDb {
    fn begin_transaction(&self) -> DatabaseTransaction {
        let mut tx: DatabaseTransaction = SledTransaction {
            operations: Vec::new(),
            db: self,
            num_pending_operations: 0,
            num_savepoint_operations: 0,
        }
        .into();
        tx.set_tx_savepoint();
        tx
    }
}

// Sled database transaction should only be used for test code and never for production
// as it doesn't properly implement MVCC
#[async_trait(?Send)]
impl<'a> IDatabaseTransaction<'a> for SledTransaction<'a> {
    fn raw_insert_bytes(&mut self, key: &[u8], value: Vec<u8>) -> Result<Option<Vec<u8>>> {
        let val = self.raw_get_bytes(key);
        self.operations
            .push(DatabaseOperation::Insert(DatabaseInsertOperation {
                key: key.to_vec(),
                value,
            }));
        self.num_pending_operations += 1;
        val
    }

    fn raw_get_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let mut val: Option<Vec<u8>> = None;
        let mut deleted = false;
        // First iterate through pending writes to support "read our own writes"
        for op in &self.operations {
            match op {
                DatabaseOperation::Insert(insert_op) if insert_op.key == key => {
                    deleted = false;
                    val = Some(insert_op.value.clone());
                }
                DatabaseOperation::Delete(delete_op) if delete_op.key == key => {
                    deleted = true;
                }
                _ => {}
            }
        }

        if deleted {
            return Ok(None);
        }

        if val.is_some() {
            return Ok(val);
        }

        Ok(self
            .db
            .inner()
            .get(key)
            .map_err(anyhow::Error::from)?
            .map(|bytes| bytes.to_vec()))
    }

    fn raw_remove_entry(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let ret = self.raw_get_bytes(key);
        self.operations
            .push(DatabaseOperation::Delete(DatabaseDeleteOperation {
                key: key.to_vec(),
            }));
        self.num_pending_operations += 1;
        ret
    }

    fn raw_find_by_prefix(&self, key_prefix: &[u8]) -> PrefixIter<'_> {
        let mut val = BTreeMap::new();
        // First iterate through pending writes to support "read our own writes"
        for op in &self.operations {
            match op {
                DatabaseOperation::Insert(insert_op) if insert_op.key.starts_with(key_prefix) => {
                    val.insert(
                        insert_op.key.clone(),
                        Ok((insert_op.key.clone(), insert_op.value.clone())),
                    );
                }
                DatabaseOperation::Delete(delete_op)
                    if delete_op.key.starts_with(key_prefix)
                        && val.contains_key(&delete_op.key) =>
                {
                    val.remove(&delete_op.key);
                }
                _ => {}
            }
        }

        let mut dbscan: Vec<Result<_, anyhow::Error>> = self
            .db
            .inner()
            .scan_prefix(key_prefix)
            .map(|res| {
                res.map(|(key_bytes, value_bytes)| (key_bytes.to_vec(), value_bytes.to_vec()))
                    .map_err(anyhow::Error::from)
            })
            .collect();

        let mut values: Vec<Result<_, anyhow::Error>> = val.into_values().collect();
        dbscan.append(&mut values);
        Box::new(dbscan.into_iter())
    }

    async fn commit_tx(self: Box<Self>) -> Result<()> {
        let ret = self
            .db
            .inner()
            .transaction::<_, _, TransactionError>(|t| {
                for op in self.operations.iter() {
                    match op {
                        DatabaseOperation::Insert(insert_op) => {
                            t.insert(insert_op.key.clone(), insert_op.value.clone())?;
                        }
                        DatabaseOperation::Delete(delete_op) => {
                            t.remove(delete_op.key.clone())?;
                        }
                    }
                }
                Ok(())
            })
            .map_err(anyhow::Error::from);

        self.db.inner().flush().expect("DB failure");
        ret
    }

    fn rollback_tx_to_savepoint(&mut self) {
        // Remove any pending operations beyond the savepoint
        let removed_ops = self.num_pending_operations - self.num_savepoint_operations;
        for _i in 0..removed_ops {
            self.operations.pop();
        }
    }

    fn set_tx_savepoint(&mut self) {
        self.num_savepoint_operations = self.num_pending_operations;
    }
}

#[cfg(test)]
mod fedimint_sled_tests {
    use crate::SledDb;

    fn open_temp_db(temp_path: &str) -> SledDb {
        let path = tempfile::Builder::new()
            .prefix(temp_path)
            .tempdir()
            .unwrap();
        SledDb::open(path, "default").unwrap()
    }

    #[test_log::test(tokio::test)]
    async fn test_dbtx_insert_elements() {
        fedimint_api::db::verify_insert_elements(
            open_temp_db("fcb-sled-test-insert-elements").into(),
        )
        .await;
    }

    #[test_log::test]
    fn test_dbtx_remove_nonexisting() {
        fedimint_api::db::verify_remove_nonexisting(
            open_temp_db("fcb-sled-test-remove-nonexisting").into(),
        );
    }

    #[test_log::test]
    fn test_dbtx_remove_existing() {
        fedimint_api::db::verify_remove_existing(
            open_temp_db("fcb-sled-test-remove-existing").into(),
        );
    }

    #[test_log::test]
    fn test_dbtx_read_own_writes() {
        fedimint_api::db::verify_read_own_writes(
            open_temp_db("fcb-sled-test-read-own-writes").into(),
        );
    }

    #[test_log::test]
    fn test_dbtx_prevent_dirty_reads() {
        fedimint_api::db::verify_prevent_dirty_reads(
            open_temp_db("fcb-sled-test-prevent-dirty-reads").into(),
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_dbtx_find_by_prefix() {
        fedimint_api::db::verify_find_by_prefix(
            open_temp_db("fcb-sled-test-find-by-prefix").into(),
        )
        .await;
    }

    #[test_log::test(tokio::test)]
    async fn test_dbtx_commit() {
        fedimint_api::db::verify_commit(open_temp_db("fcb-sled-test-commit").into()).await;
    }

    #[test_log::test]
    fn test_dbtx_rollback_to_savepoint() {
        fedimint_api::db::verify_rollback_to_savepoint(
            open_temp_db("fcb-sled-test-rollback-to-savepoint").into(),
        );
    }
}
