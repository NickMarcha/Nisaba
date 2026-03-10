// Index Markdown docs for blocks and links. Ported from desktop indexer.

use regex::Regex;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexedBlock {
    pub id: String,
    pub file_path: String,
    pub block_index: i32,
    pub block_type: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexedLink {
    pub source_file: String,
    pub target: String,
    pub relation_key: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Frontmatter {
    #[serde(rename = "block_ids")]
    block_ids: Option<Vec<String>>,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

fn extract_wikilink_target(s: &str) -> Option<String> {
    let re = Regex::new(r"^\[\[([^\]|]+)(?:\|[^\]]*)?\]\]$").unwrap();
    re.captures(s.trim())
        .map(|c| c.get(1).unwrap().as_str().to_string())
}

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
        let raw: serde_yaml::Value =
            serde_yaml::from_str(yaml_str).unwrap_or(serde_yaml::Value::Null);
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

fn split_blocks(body: &str) -> Vec<String> {
    body.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Index a set of (doc_id, content) pairs. Returns blocks and links.
pub fn index_docs(
    docs: &[(String, String)],
) -> (Vec<IndexedBlock>, Vec<IndexedLink>) {
    let mut blocks = Vec::new();
    let mut links = Vec::new();

    for (path_str, content) in docs {
        let (frontmatter, raw_fm, body) = parse_frontmatter(content);

        if let Some(ref raw) = raw_fm {
            for (key, target) in extract_frontmatter_relations(raw) {
                links.push(IndexedLink {
                    source_file: path_str.clone(),
                    target,
                    relation_key: Some(key),
                });
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

        let block_contents = split_blocks(body);
        for (i, block_content) in block_contents.iter().enumerate() {
            let id = block_ids
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("{}:{}", path_str, i));
            blocks.push(IndexedBlock {
                id,
                file_path: path_str.clone(),
                block_index: i as i32,
                block_type: block_type.clone(),
                content: block_content.clone(),
            });
        }

        let wikilinks = extract_wikilinks(content);
        for target in wikilinks {
            links.push(IndexedLink {
                source_file: path_str.clone(),
                target,
                relation_key: None,
            });
        }
    }

    (blocks, links)
}

/// Serialize index to JSON for Yjs storage.
pub fn index_to_json(blocks: &[IndexedBlock], links: &[IndexedLink]) -> String {
    let blocks_json: Vec<serde_json::Value> = blocks
        .iter()
        .map(|b| {
            json!({
                "id": b.id,
                "file_path": b.file_path,
                "block_index": b.block_index,
                "block_type": b.block_type,
                "content": b.content,
            })
        })
        .collect();
    let links_json: Vec<serde_json::Value> = links
        .iter()
        .map(|l| {
            json!({
                "source_file": l.source_file,
                "target": l.target,
                "relation_key": l.relation_key,
            })
        })
        .collect();
    serde_json::json!({ "blocks": blocks_json, "links": links_json }).to_string()
}
