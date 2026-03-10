// SQLite persistence for Yjs document state.

use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Arc;
use tracing::error;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS yjs_docs (
    doc_id TEXT PRIMARY KEY,
    state BLOB NOT NULL,
    updated_at INTEGER NOT NULL
);
"#;

pub struct Persistence {
    conn: Arc<std::sync::Mutex<Connection>>,
}

impl Persistence {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    pub fn list_doc_ids(&self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.conn.lock().map_err(|_| {
            rusqlite::Error::InvalidParameterName("lock poisoned".to_string())
        })?;
        let mut stmt = conn.prepare("SELECT doc_id FROM yjs_docs ORDER BY doc_id")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let ids: Result<Vec<String>, _> = rows.collect();
        ids
    }

    pub fn load(&self, doc_id: &str) -> Result<Option<Vec<u8>>, rusqlite::Error> {
        let conn = self.conn.lock().map_err(|_| {
            rusqlite::Error::InvalidParameterName("lock poisoned".to_string())
        })?;
        let mut stmt = conn.prepare("SELECT state FROM yjs_docs WHERE doc_id = ?1")?;
        let mut rows = stmt.query(params![doc_id])?;
        if let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            Ok(Some(blob))
        } else {
            Ok(None)
        }
    }

    pub fn save(&self, doc_id: &str, state: &[u8]) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().map_err(|_| {
            rusqlite::Error::InvalidParameterName("lock poisoned".to_string())
        })?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        conn.execute(
            "INSERT INTO yjs_docs (doc_id, state, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(doc_id) DO UPDATE SET state = ?2, updated_at = ?3",
            params![doc_id, state, now],
        )?;
        Ok(())
    }
}

pub fn spawn_persist(
    persistence: Arc<Persistence>,
    doc_id: String,
    state: Vec<u8>,
) {
    std::thread::spawn(move || {
        if let Err(e) = persistence.save(&doc_id, &state) {
            error!("Failed to persist doc {}: {}", doc_id, e);
        } else {
            tracing::info!("Persisted doc {} ({} bytes)", doc_id, state.len());
        }
    });
}
