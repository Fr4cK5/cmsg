use std::{cell::RefCell, fs, path::Path};

use eyre::{OptionExt, Result, eyre};
use rusqlite::{Connection, Transaction, named_params};

use crate::{config::StorageStrategy, meta::types::Backup};

pub mod types;

/// A wrapper around a `rusqlite::Connection`.
///
/// This also houses some functions to abstract away some common SQL operations.
pub struct MetadataRepo {
    connection: RefCell<Connection>,
}

impl MetadataRepo {
    pub fn new() -> Result<Self> {
        let db_file = StorageStrategy::user_home_db()?;
        let path = db_file.parent().ok_or_eyre(eyre!(
            "Unable to get parent directory of given database path '{}'",
            &db_file.display()
        ))?;

        fs::create_dir_all(path)?;

        let conn = Connection::open(&db_file)?;
        conn.execute_batch(crate::CREATE_DB)
            .map_err(|err| eyre!("{err}: DB initialization failed."))?;

        Ok(Self {
            connection: RefCell::new(conn),
        })
    }

    pub fn upsert_path(tx: &Transaction, path: &Path) -> Result<i64> {
        let mut prepared =
            tx.prepare("insert or ignore into data_directory (path) values (:path)")?;

        match prepared.insert(named_params! {
            ":path": path.to_str(),
        }) {
            Ok(row_id) => Ok(row_id),
            Err(_) => {
                let mut prepared =
                    tx.prepare("select id from data_directory where path = :path")?;
                Ok(prepared.query_one::<i64, _, _>(
                    named_params! {
                        ":path": path.to_str(),
                    },
                    |item| item.get(0),
                )?)
            }
        }
    }

    pub fn insert_backup_record(
        tx: &Transaction,
        backup: &Backup,
        data_directory_id: i64,
    ) -> Result<i64> {
        let mut prepared = tx.prepare(
            "insert into backup_entry (hash, data_directory_id) values (:hash, :dir_id)",
        )?;

        Ok(prepared.insert(named_params! {
            ":hash": &backup.backup_hash,
            ":dir_id": data_directory_id,
        })?)
    }

    pub fn fetch_hashes_for_data_directory(
        tx: &Transaction,
        data_dir: &Path,
    ) -> Result<Option<(Vec<String>, i64)>> {
        let mut prepared = tx.prepare("select id from data_directory where path = :path")?;
        let Ok(id) = prepared.query_one(
            named_params! {
                ":path": data_dir.to_str()
            },
            |row| row.get::<_, i64>("id"),
        ) else {
            return Ok(None);
        };

        let mut prepared =
            tx.prepare("select hash from backup_entry where data_directory_id = :dd_id")?;
        let commit_hashes = prepared
            .query_map(
                named_params! {
                    ":dd_id": id,
                },
                |row| row.get::<_, String>(0),
            )?
            .flatten();

        let mut rows = Vec::new();
        for row in commit_hashes {
            rows.push(row);
        }

        Ok(Some((rows, id)))
    }

    pub fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: Fn(&Transaction) -> Result<T>,
    {
        let mut conn = self.connection.borrow_mut();
        let tx = conn.transaction()?;
        let result = f(&tx);
        match &result {
            Ok(_) => tx.commit()?,
            Err(_) => tx.rollback()?,
        }
        result
    }
}
