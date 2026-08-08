use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use rusqlite::{Connection, Transaction, named_params};

use crate::{config::StorageStrategy, meta::types::Backup};

pub mod types;

pub struct MetadataRepo {
    pub connection: RefCell<Connection>,
}

impl MetadataRepo {
    pub fn new() -> Result<Self> {
        let path = StorageStrategy::Global
            .locate_data_dir()
            .unwrap_or_else(|| PathBuf::from("."));

        fs::create_dir_all(&path)?;
        let db_file = path.join("data.db");

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
            "insert into backup_entry (digest, data_directory_id) values (:digest, :dir_id)",
        )?;

        Ok(prepared.insert(named_params! {
            ":digest": &backup.digest,
            ":dir_id": data_directory_id,
        })?)
    }
}
