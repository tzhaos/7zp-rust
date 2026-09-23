use super::recent::{Entry, Kind};
use anyhow::{Context, Result};
use cardo_runtime::database::{
    Database,
    rusqlite::{self, OptionalExtension, params},
};
use serde::{Serialize, de::DeserializeOwned};

const SCHEMA: &str = "
CREATE TABLE state (key TEXT PRIMARY KEY, value TEXT NOT NULL CHECK(json_valid(value))) STRICT;
CREATE TABLE history (
    position INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK(kind IN ('archive', 'folder'))
) STRICT;
";

pub struct StateStore {
    pub(crate) database: Database,
}

impl StateStore {
    pub(crate) fn open() -> Result<Self> {
        Ok(Self {
            database: Database::open(
                super::directory()?.join("state.sqlite3"),
                0x50375a31,
                1,
                SCHEMA,
            )?,
        })
    }

    pub fn read_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        self.database.read(|connection| {
            let value: Option<String> = connection
                .query_row("SELECT value FROM state WHERE key = ?1", [key], |row| {
                    row.get(0)
                })
                .optional()?;
            value
                .map(|value| {
                    serde_json::from_str(&value)
                        .with_context(|| format!("Invalid stored value for {key}"))
                })
                .transpose()
        })
    }

    pub fn write_json(&self, key: &str, value: &impl Serialize) -> Result<()> {
        self.database
            .write(|transaction| put(transaction, key, value))
    }

    pub fn remove(&self, key: &str) -> Result<()> {
        self.database.write(|transaction| {
            transaction.execute("DELETE FROM state WHERE key = ?1", [key])?;
            Ok(())
        })
    }

    pub fn finish_update(&self, result: &impl Serialize) -> Result<()> {
        self.database.write(|transaction| {
            put(transaction, "update-result", result)?;
            transaction.execute("DELETE FROM state WHERE key = 'update-pending'", [])?;
            Ok(())
        })
    }
}

fn put(connection: &rusqlite::Connection, key: &str, value: &impl Serialize) -> Result<()> {
    let value =
        serde_json::to_string(value).with_context(|| format!("Cannot serialize state {key}"))?;
    connection.execute("INSERT INTO state(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, value])?;
    Ok(())
}

pub(crate) fn remember(connection: &rusqlite::Connection, entry: &Entry) -> Result<()> {
    let path = entry
        .path
        .to_str()
        .context("History path cannot be represented as UTF-8")?;
    connection.execute("DELETE FROM history WHERE path = ?1", [path])?;
    connection.execute(
        "INSERT INTO history(path,kind) VALUES (?1,?2)",
        params![
            path,
            match entry.kind {
                Kind::Archives => "archive",
                Kind::Folders => "folder",
            }
        ],
    )?;
    Ok(())
}

pub(crate) fn history(connection: &rusqlite::Connection) -> Result<Vec<Entry>> {
    let mut statement =
        connection.prepare("SELECT path,kind FROM history ORDER BY position DESC LIMIT 20")?;
    let rows = statement.query_map([], |row| {
        let kind: String = row.get(1)?;
        Ok(Entry {
            path: row.get::<_, String>(0)?.into(),
            kind: match kind.as_str() {
                "archive" => Kind::Archives,
                "folder" => Kind::Folders,
                _ => return Err(rusqlite::Error::InvalidQuery),
            },
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
