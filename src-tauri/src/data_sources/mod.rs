use crate::volumes::repository::online_volume_ids;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceRecord {
    pub id: i64,
    pub volume_id: i64,
    pub root_path: String,
    pub label: Option<String>,
    pub enabled: bool,
    pub volume_label: String,
    pub drive_letter: Option<String>,
    pub volume_online: bool,
}

pub fn list_data_sources(conn: &Connection) -> Result<Vec<DataSourceRecord>, String> {
    let online: std::collections::HashSet<i64> = online_volume_ids(conn)
        .unwrap_or_default()
        .into_iter()
        .collect();

    let mut stmt = conn
        .prepare(
            "SELECT ds.id, ds.volume_id, ds.root_path, ds.label, ds.enabled,
                    v.label, v.last_drive_letter
             FROM data_sources ds
             JOIN volumes v ON v.id = ds.volume_id
             ORDER BY v.last_drive_letter, ds.root_path",
        )
        .map_err(|e| format!("Failed to prepare data sources query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let volume_id: i64 = row.get(1)?;
            Ok(DataSourceRecord {
                id: row.get(0)?,
                volume_id,
                root_path: row.get(2)?,
                label: row.get(3)?,
                enabled: row.get::<_, i64>(4)? == 1,
                volume_label: row.get(5)?,
                drive_letter: row.get(6)?,
                volume_online: online.contains(&volume_id),
            })
        })
        .map_err(|e| format!("Failed to query data sources: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read data sources: {e}"))
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDataSourceResult {
    pub id: i64,
    pub created: bool,
    pub nested_warning: Option<String>,
}

pub fn add_data_source(
    conn: &Connection,
    volume_id: i64,
    root_path: &str,
    label: Option<&str>,
) -> Result<AddDataSourceResult, String> {
    let normalized = normalize_relative_path(root_path);

    if let Some(existing_id) = conn
        .query_row(
            "SELECT id FROM data_sources WHERE volume_id = ?1 AND root_path = ?2",
            params![volume_id, normalized],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|e| format!("Failed to look up existing data source: {e}"))?
    {
        if let Some(new_label) = label {
            conn.execute(
                "UPDATE data_sources SET label = ?1, enabled = 1, updated_at = datetime('now') WHERE id = ?2",
                params![new_label, existing_id],
            )
            .map_err(|e| format!("Failed to refresh data source: {e}"))?;
        } else {
            conn.execute(
                "UPDATE data_sources SET enabled = 1, updated_at = datetime('now') WHERE id = ?1",
                params![existing_id],
            )
            .map_err(|e| format!("Failed to refresh data source: {e}"))?;
        }

        return Ok(AddDataSourceResult {
            id: existing_id,
            created: false,
            nested_warning: nested_data_source_warning(
                conn,
                volume_id,
                &normalized,
                existing_id,
            )?,
        });
    }

    conn.execute(
        "INSERT INTO data_sources (volume_id, root_path, label) VALUES (?1, ?2, ?3)",
        params![volume_id, normalized, label],
    )
    .map_err(|e| format!("Failed to add data source: {e}"))?;

    let new_id = conn.last_insert_rowid();
    Ok(AddDataSourceResult {
        id: new_id,
        created: true,
        nested_warning: nested_data_source_warning(conn, volume_id, &normalized, new_id)?,
    })
}

fn path_is_prefix(parent: &str, child: &str) -> bool {
    let parent = parent.trim_end_matches('\\').to_ascii_lowercase();
    let child = child.trim_end_matches('\\').to_ascii_lowercase();
    if parent == child {
        return false;
    }
    child.starts_with(&(parent + "\\"))
}

fn nested_data_source_warning(
    conn: &Connection,
    volume_id: i64,
    new_path: &str,
    self_id: i64,
) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT id, root_path FROM data_sources WHERE volume_id = ?1")
        .map_err(|e| format!("Failed to check nested data sources: {e}"))?;

    let rows = stmt
        .query_map(params![volume_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to query nested data sources: {e}"))?;

    for row in rows {
        let (id, existing) = row.map_err(|e| format!("Failed to read nested data source: {e}"))?;
        if id == self_id {
            continue;
        }
        if path_is_prefix(&existing, new_path) {
            return Ok(Some(format!(
                "{new_path} is inside existing source {existing}. Scan the parent folder only, or remove the parent entry."
            )));
        }
        if path_is_prefix(new_path, &existing) {
            return Ok(Some(format!(
                "Existing source {existing} is inside {new_path}. Scanning both may duplicate work — prefer one root per drive."
            )));
        }
    }

    Ok(None)
}

pub fn add_data_source_from_absolute_path(
    conn: &Connection,
    absolute_path: &str,
    label: Option<&str>,
) -> Result<AddDataSourceResult, String> {
    let path = absolute_path.trim().replace('/', "\\");
    let drive_letter = parse_drive_letter(&path)
        .ok_or_else(|| "Path must start with a drive letter (e.g. E:\\Movies)".to_string())?;

    let volume_id: i64 = conn
        .query_row(
            "SELECT id FROM volumes WHERE last_drive_letter = ?1",
            params![drive_letter],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Volume lookup failed: {e}"))?
        .ok_or_else(|| {
            format!(
                "No known volume for {drive_letter}. Sync connected drives first."
            )
        })?;

    let relative = absolute_to_relative(&drive_letter, Path::new(&path))?;
    add_data_source(conn, volume_id, &relative, label)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDataSourceResult {
    pub movies_removed: usize,
    pub root_path: String,
}

pub fn remove_data_source(conn: &Connection, id: i64) -> Result<RemoveDataSourceResult, String> {
    let (volume_id, root_path): (i64, String) = conn
        .query_row(
            "SELECT volume_id, root_path FROM data_sources WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| format!("Data source {id} not found"))?;

    let root_trimmed = root_path.trim_end_matches('\\');
    let under_prefix = format!("{root_trimmed}\\%");
    let removed_movies = conn
        .execute(
            "DELETE FROM movies
             WHERE data_source_id = ?1
                OR (
                    volume_id = ?2
                    AND (relative_path = ?3 OR relative_path LIKE ?4 ESCAPE '\\')
                )",
            params![id, volume_id, root_path, under_prefix],
        )
        .map_err(|e| format!("Failed to remove indexed movies: {e}"))?;

    let removed = conn
        .execute("DELETE FROM data_sources WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to remove data source: {e}"))?;
    if removed == 0 {
        return Err(format!("Data source {id} not found"));
    }

    let _ = crate::duplicates::rebuild_duplicate_groups(conn);

    Ok(RemoveDataSourceResult {
        movies_removed: removed_movies,
        root_path,
    })
}

pub fn resolve_data_source_root(
    conn: &Connection,
    data_source_id: i64,
) -> Result<(String, i64), String> {
    conn.query_row(
        "SELECT v.last_drive_letter, ds.volume_id
         FROM data_sources ds
         JOIN volumes v ON v.id = ds.volume_id
         WHERE ds.id = ?1",
        params![data_source_id],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get(1)?)),
    )
    .optional()
    .map_err(|e| format!("Failed to resolve data source: {e}"))?
    .and_then(|(letter, volume_id)| letter.map(|value| (value, volume_id)))
    .ok_or_else(|| "Data source volume is offline or missing a drive letter".to_string())
}

fn parse_drive_letter(path: &str) -> Option<String> {
    let chars: Vec<char> = path.chars().collect();
    if chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
        return Some(format!("{}:", chars[0].to_ascii_uppercase()));
    }
    None
}

fn absolute_to_relative(drive_letter: &str, path: &Path) -> Result<String, String> {
    let mut normalized = path.to_string_lossy().replace('/', "\\");
    if !normalized.to_ascii_uppercase().starts_with(&format!(
        "{}:",
        drive_letter.chars().next().unwrap_or('?')
    )) {
        return Err("Absolute path does not match selected drive".to_string());
    }

    if let Some(stripped) = normalized.get(2..) {
        normalized = stripped.trim_start_matches('\\').to_string();
    }

    Ok(normalize_relative_path(&normalized))
}

fn normalize_relative_path(path: &str) -> String {
    let trimmed = path.trim().replace('/', "\\");
    if trimmed.starts_with('\\') {
        trimmed
    } else {
        format!("\\{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;

    #[test]
    fn normalize_path_adds_leading_backslash() {
        assert_eq!(normalize_relative_path("Movies/4K"), "\\Movies\\4K");
    }

    #[test]
    fn remove_data_source_drops_movies_under_root() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn.execute(
            "INSERT INTO volumes (serial, guid, label, filesystem, last_drive_letter)
             VALUES ('AA', 'GUID', 'Test', 'NTFS', 'F:')",
            [],
        )
        .unwrap();
        let volume_id = conn.last_insert_rowid();
        let source_id = add_data_source(&conn, volume_id, r"\Movies\1080p", None)
            .unwrap()
            .id;
        conn.execute(
            "INSERT INTO movies (title, relative_path, volume_id, data_source_id)
             VALUES ('In folder', '\\Movies\\1080p\\Title', ?1, ?2)",
            params![volume_id, source_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO movies (title, relative_path, volume_id, data_source_id)
             VALUES ('Other', '\\Movies\\4K\\Title', ?1, NULL)",
            params![volume_id],
        )
        .unwrap();

        let result = remove_data_source(&conn, source_id).unwrap();
        assert_eq!(result.movies_removed, 1);

        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM movies", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 1);
    }

    #[test]
    fn add_data_source_roundtrip() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn.execute(
            "INSERT INTO volumes (serial, guid, label, filesystem, last_drive_letter)
             VALUES ('AA', 'GUID', 'Test', 'NTFS', 'E:')",
            [],
        )
        .unwrap();
        let volume_id = conn.last_insert_rowid();
        let id = add_data_source(&conn, volume_id, "Movies", Some("Main movies"))
            .unwrap()
            .id;
        let rows = list_data_sources(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id);
        assert_eq!(rows[0].root_path, "\\Movies");
    }
}