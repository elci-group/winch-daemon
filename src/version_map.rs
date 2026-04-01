use rusqlite::{Connection, params};
use anyhow::Result;

pub struct VersionMap {
    conn: Connection,
}

impl VersionMap {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS crate_versions (
                crate_name TEXT,
                version TEXT,
                last_success TEXT,
                project TEXT
            )", []
        )?;
        Ok(Self { conn })
    }

    pub fn record_version(&self, crate_name: &str, version: &str, project: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO crate_versions (crate_name, version, last_success, project)
             VALUES (?1, ?2, datetime('now'), ?3)",
            params![crate_name, version, project],
        )?;
        Ok(())
    }
}
