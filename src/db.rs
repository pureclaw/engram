/// SQLite + sqlite-vec database layer.
///
/// Schema:
///   documents  — one row per indexed file (path, hash, snippet, timestamps)
///   chunks     — one row per chunk of text (virtual vec0 table for ANN search)
///   meta       — key/value store for index metadata (provider name, dims, etc.)
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Db {
    pub conn: Connection,
}

impl Db {
    pub fn open(db_path: &Path) -> Result<Self> {
        // Register sqlite-vec as a global auto-extension before opening any connection.
        // Safe to call multiple times — SQLite deduplicates by function pointer.
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
                *const (),
                unsafe extern "C" fn(
                    *mut rusqlite::ffi::sqlite3,
                    *mut *const std::ffi::c_char,
                    *const rusqlite::ffi::sqlite3_api_routines,
                ) -> i32,
            >(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

        Ok(Db { conn })
    }

    pub fn init(&self, dims: usize, provider_name: &str) -> Result<()> {
        self.conn
            .execute_batch(&format!(
                "
            CREATE TABLE IF NOT EXISTS documents (
                id      INTEGER PRIMARY KEY,
                path    TEXT NOT NULL UNIQUE,
                hash    TEXT NOT NULL,
                snippet TEXT NOT NULL,
                indexed_at INTEGER NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS chunks USING vec0(
                document_id INTEGER,
                embedding   FLOAT[{dims}]
            );

            CREATE TABLE IF NOT EXISTS meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            INSERT OR IGNORE INTO meta (key, value) VALUES ('provider', '{provider_name}');
            INSERT OR IGNORE INTO meta (key, value) VALUES ('dims', '{dims}');
            "
            ))
            .context("Failed to initialize schema")
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn upsert_document(&self, path: &str, hash: &str, snippet: &str) -> Result<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO documents (path, hash, snippet, indexed_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET hash=excluded.hash,
               snippet=excluded.snippet, indexed_at=excluded.indexed_at",
            params![path, hash, snippet, now],
        )?;

        let id: i64 = self.conn.query_row(
            "SELECT id FROM documents WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )?;

        // Remove old embedding chunks for this document
        self.conn
            .execute("DELETE FROM chunks WHERE document_id = ?1", params![id])?;

        Ok(id)
    }

    pub fn insert_chunk(&self, document_id: i64, embedding: &[f32]) -> Result<()> {
        // sqlite-vec stores vectors as raw bytes (little-endian f32)
        let bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

        self.conn.execute(
            "INSERT INTO chunks (document_id, embedding) VALUES (?1, ?2)",
            params![document_id, bytes],
        )?;
        Ok(())
    }

    pub fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let bytes: Vec<u8> = query_embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        let mut stmt = self.conn.prepare(
            "SELECT d.path, d.snippet, c.distance
             FROM chunks c
             JOIN documents d ON c.document_id = d.id
             WHERE c.embedding MATCH ?1
               AND k = ?2
             ORDER BY c.distance",
        )?;

        let results = stmt
            .query_map(params![bytes, limit as i64], |row| {
                Ok(SearchResult {
                    path: row.get(0)?,
                    snippet: row.get(1)?,
                    distance: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(results)
    }

    pub fn remove_document(&self, path: &str) -> Result<bool> {
        let id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM documents WHERE path = ?1",
                params![path],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = id {
            self.conn
                .execute("DELETE FROM chunks WHERE document_id = ?1", params![id])?;
            self.conn
                .execute("DELETE FROM documents WHERE id = ?1", params![id])?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn document_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_hash(&self, path: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT hash FROM documents WHERE path = ?1",
            params![path],
            |row| row.get(0),
        );
        match result {
            Ok(h) => Ok(Some(h)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn all_paths(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM documents")?;
        let paths = stmt
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(paths)
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub path: String,
    pub snippet: String,
    pub distance: f32,
}
