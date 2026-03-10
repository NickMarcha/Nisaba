use std::fs;
use std::path::Path;
use tauri::command;
use walkdir::WalkDir;

use crate::indexer;
use crate::watcher;

#[derive(serde::Serialize)]
pub struct IndexedBlock {
    pub id: String,
    pub file_path: String,
    pub block_index: i32,
    pub block_type: Option<String>,
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct IndexedLink {
    pub source_file: String,
    pub target: String,
    /// Present for frontmatter relations (e.g. "assignee"); null for body wikilinks.
    pub relation_key: Option<String>,
}

#[command]
pub fn open_vault() -> Option<String> {
    rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().into_owned())
}

#[command]
pub fn list_vault_files(vault_path: String) -> Result<Vec<String>, String> {
    let path = Path::new(&vault_path);
    if !path.is_dir() {
        return Err("Path is not a directory".into());
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().map_or(false, |e| e == "md") {
                files.push(p.to_string_lossy().into_owned());
            }
        }
    }
    files.sort();
    Ok(files)
}

#[command]
pub fn read_file(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    if !p.is_file() {
        return Err("Path is not a file".into());
    }
    fs::read_to_string(p).map_err(|e| e.to_string())
}

#[command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    let p = Path::new(&path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(p, content).map_err(|e| e.to_string())
}

#[command]
pub fn rename_file(old_path: String, new_path: String) -> Result<(), String> {
    let old_p = Path::new(&old_path);
    let new_p = Path::new(&new_path);
    if !old_p.is_file() {
        return Err("Source file does not exist".into());
    }
    if new_p.exists() {
        return Err("Target file already exists".into());
    }
    if let Some(parent) = new_p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::rename(old_p, new_p).map_err(|e| e.to_string())
}

#[command]
pub fn index_vault(vault_path: String) -> Result<(), String> {
    indexer::index_vault(Path::new(&vault_path))
        .map(|_| ())
}

#[command]
pub fn query_blocks(vault_path: String, block_type: Option<String>) -> Result<Vec<IndexedBlock>, String> {
    let db_path = Path::new(&vault_path).join("index.db");
    if !db_path.exists() {
        return Ok(Vec::new());
    }
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    let blocks: Vec<IndexedBlock> = match block_type {
        Some(t) => conn
            .prepare("SELECT id, file_path, block_index, type, content FROM blocks WHERE type = ?1 ORDER BY file_path, block_index")
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![t], |row| {
                Ok(IndexedBlock {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    block_index: row.get(2)?,
                    block_type: row.get(3)?,
                    content: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        None => conn
            .prepare("SELECT id, file_path, block_index, type, content FROM blocks ORDER BY file_path, block_index")
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(IndexedBlock {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    block_index: row.get(2)?,
                    block_type: row.get(3)?,
                    content: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    };
    Ok(blocks)
}

#[command]
pub fn watch_vault(
    app: tauri::AppHandle,
    vault_path: String,
    state: tauri::State<'_, crate::VaultWatcherState>,
) -> Result<(), String> {
    watcher::watch_vault(app, Path::new(&vault_path), state.stop_tx.clone())
}

#[command]
pub fn query_links(
    vault_path: String,
    source_file: Option<String>,
    target: Option<String>,
) -> Result<Vec<IndexedLink>, String> {
    let db_path = Path::new(&vault_path).join("index.db");
    if !db_path.exists() {
        return Ok(Vec::new());
    }
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;

    let mut links = Vec::new();

    // Body wikilinks
    let link_rows: Vec<IndexedLink> = match (&source_file, &target) {
        (Some(s), None) => conn
            .prepare("SELECT source_file, target FROM links WHERE source_file = ?1")
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(1)?,
                    relation_key: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, Some(t)) => conn
            .prepare("SELECT source_file, target FROM links WHERE target = ?1")
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![t], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(1)?,
                    relation_key: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (Some(s), Some(t)) => conn
            .prepare("SELECT source_file, target FROM links WHERE source_file = ?1 AND target = ?2")
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s, t], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(1)?,
                    relation_key: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, None) => conn
            .prepare("SELECT source_file, target FROM links")
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(1)?,
                    relation_key: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    };
    links.extend(link_rows);

    // Frontmatter relations
    let rel_rows: Vec<IndexedLink> = match (&source_file, &target) {
        (Some(s), None) => conn
            .prepare("SELECT source_file, relation_key, target FROM relations WHERE source_file = ?1")
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(2)?,
                    relation_key: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, Some(t)) => conn
            .prepare("SELECT source_file, relation_key, target FROM relations WHERE target = ?1")
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![t], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(2)?,
                    relation_key: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (Some(s), Some(t)) => conn
            .prepare("SELECT source_file, relation_key, target FROM relations WHERE source_file = ?1 AND target = ?2")
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s, t], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(2)?,
                    relation_key: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, None) => conn
            .prepare("SELECT source_file, relation_key, target FROM relations")
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(IndexedLink {
                    source_file: row.get(0)?,
                    target: row.get(2)?,
                    relation_key: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    };
    links.extend(rel_rows);

    Ok(links)
}

#[command]
pub fn query_files(vault_path: String) -> Result<Vec<String>, String> {
    let db_path = Path::new(&vault_path).join("index.db");
    if !db_path.exists() {
        return Ok(Vec::new());
    }
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    let paths: Vec<String> = conn
        .prepare("SELECT path FROM files ORDER BY path")
        .map_err(|e| e.to_string())?
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(paths)
}
