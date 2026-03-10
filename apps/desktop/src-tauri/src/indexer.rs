use regex::Regex;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    #[serde(rename = "block_ids")]
    block_ids: Option<Vec<String>>,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

/// Extract wikilink target from "[[target]]" or "[[target|alias]]".
fn extract_wikilink_target(s: &str) -> Option<String> {
    let re = Regex::new(r"^\[\[([^\]|]+)(?:\|[^\]]*)?\]\]$").unwrap();
    re.captures(s.trim()).map(|c| c.get(1).unwrap().as_str().to_string())
}

/// Extract relation targets from frontmatter. Keys like assignee, project with values [[Note]].
fn extract_frontmatter_relations(fm: &serde_yaml::Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let map = match fm.as_mapping() {
        Some(m) => m,
        None => return out,
    };
    for (k, v) in map {
        let key = k.as_str().unwrap_or_default().to_string();
        if key == "block_ids" || key == "type" || key == "tags" {
            continue;
        }
        match v {
            serde_yaml::Value::String(s) => {
                if let Some(target) = extract_wikilink_target(s) {
                    out.push((key, target));
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    if let Some(s) = item.as_str() {
                        if let Some(target) = extract_wikilink_target(s) {
                            out.push((key.clone(), target));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn parse_frontmatter(content: &str) -> (Option<Frontmatter>, Option<serde_yaml::Value>, &str) {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return (None, None, content);
    }
    let rest = &content[3..];
    if let Some(end) = rest.find("\n---") {
        let yaml_str = rest[..end].trim();
        let body = rest[end + 4..].trim_start();
        let fm: Frontmatter = serde_yaml::from_str(yaml_str).unwrap_or_default();
        let raw: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap_or(serde_yaml::Value::Null);
        return (Some(fm), Some(raw), body);
    }
    (None, None, content)
}

fn extract_wikilinks(content: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]*)?\]\]").unwrap();
    re.captures_iter(content)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

fn extract_hashtags(content: &str) -> Vec<String> {
    let re = Regex::new(r"#([a-zA-Z][a-zA-Z0-9_-]*)").unwrap();
    re.captures_iter(content)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

fn split_blocks(body: &str) -> Vec<String> {
    body.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

pub fn index_vault(vault_path: &Path) -> Result<PathBuf, String> {
    let db_path = vault_path.join("index.db");
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            path TEXT PRIMARY KEY,
            mtime INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS blocks (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            block_index INTEGER NOT NULL,
            type TEXT,
            content TEXT NOT NULL,
            properties TEXT,
            FOREIGN KEY (file_path) REFERENCES files(path)
        );
        CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY,
            source_file TEXT NOT NULL,
            target TEXT NOT NULL,
            FOREIGN KEY (source_file) REFERENCES files(path)
        );
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            tag TEXT NOT NULL,
            FOREIGN KEY (file_path) REFERENCES files(path)
        );
        CREATE TABLE IF NOT EXISTS relations (
            id INTEGER PRIMARY KEY,
            source_file TEXT NOT NULL,
            relation_key TEXT NOT NULL,
            target TEXT NOT NULL,
            FOREIGN KEY (source_file) REFERENCES files(path)
        );
        CREATE INDEX IF NOT EXISTS idx_blocks_type ON blocks(type);
        CREATE INDEX IF NOT EXISTS idx_blocks_file ON blocks(file_path);
        CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_file);
        CREATE INDEX IF NOT EXISTS idx_tags_file ON tags(file_path);
        CREATE INDEX IF NOT EXISTS idx_relations_source ON relations(source_file);
        CREATE INDEX IF NOT EXISTS idx_relations_target ON relations(target);
        "#,
    )
    .map_err(|e| e.to_string())?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM blocks", []).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM links", []).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM tags", []).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM relations", []).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM files", []).map_err(|e| e.to_string())?;

    for entry in walkdir::WalkDir::new(vault_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if p.extension().map_or(true, |e| e != "md") {
            continue;
        }
        let path_str = p.to_string_lossy().into_owned();
        let mtime = fs::metadata(p)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        tx.execute("INSERT OR REPLACE INTO files (path, mtime) VALUES (?1, ?2)", params![path_str, mtime])
            .map_err(|e| e.to_string())?;

        let content = fs::read_to_string(p).map_err(|e| e.to_string())?;
        let (frontmatter, raw_fm, body) = parse_frontmatter(&content);

        if let Some(ref raw) = raw_fm {
            for (key, target) in extract_frontmatter_relations(raw) {
                tx.execute(
                    "INSERT INTO relations (source_file, relation_key, target) VALUES (?1, ?2, ?3)",
                    params![path_str, key, target],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        let block_ids = frontmatter
            .as_ref()
            .and_then(|f| f.block_ids.as_ref())
            .cloned()
            .unwrap_or_default();
        let block_type = frontmatter
            .as_ref()
            .and_then(|f| f.r#type.as_ref())
            .cloned();

        let blocks = split_blocks(body);
        for (i, block_content) in blocks.iter().enumerate() {
            let id = block_ids.get(i).cloned().unwrap_or_else(|| format!("{}:{}", path_str, i));
            tx.execute(
                "INSERT INTO blocks (id, file_path, block_index, type, content, properties) VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
                params![id, path_str, i as i32, block_type, block_content],
            )
            .map_err(|e| e.to_string())?;
        }

        let links = extract_wikilinks(&content);
        for target in links {
            tx.execute("INSERT INTO links (source_file, target) VALUES (?1, ?2)", params![path_str, target])
                .map_err(|e| e.to_string())?;
        }

        let mut tags = extract_hashtags(&content);
        if let Some(ref fm) = frontmatter {
            if let Some(ref fm_tags) = fm.tags {
                tags.extend(fm_tags.clone());
            }
        }
        let tags: std::collections::HashSet<_> = tags.into_iter().collect();
        for tag in tags {
            tx.execute("INSERT INTO tags (file_path, tag) VALUES (?1, ?2)", params![path_str, tag])
                .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(db_path)
}
