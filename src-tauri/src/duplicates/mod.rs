use rusqlite::{params, Connection};

pub fn rebuild_duplicate_groups(conn: &Connection) -> Result<(), String> {
    conn.execute("UPDATE movies SET duplicate_group_id = NULL", [])
        .map_err(|e| format!("Failed to clear duplicate groups: {e}"))?;
    conn.execute("DELETE FROM duplicate_groups", [])
        .map_err(|e| format!("Failed to reset duplicate groups: {e}"))?;

    assign_by_key(conn, "imdb", "imdb_id")?;
    assign_by_key(conn, "tmdb", "tmdb_id")?;
    assign_by_title_year(conn)?;

    Ok(())
}

fn assign_by_key(conn: &Connection, key_type: &str, column: &str) -> Result<(), String> {
    let sql = format!(
        "SELECT {column}, GROUP_CONCAT(id)
         FROM movies
         WHERE {column} IS NOT NULL AND TRIM({column}) != ''
         GROUP BY {column}
         HAVING COUNT(*) > 1"
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare duplicate query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to query duplicates: {e}"))?;

    for row in rows {
        let (key, ids_csv) = row.map_err(|e| format!("Failed to read duplicate row: {e}"))?;
        let group_id = insert_group(conn, &key, key_type)?;
        for id in ids_csv.split(',') {
            if let Ok(movie_id) = id.trim().parse::<i64>() {
                conn.execute(
                    "UPDATE movies SET duplicate_group_id = ?1 WHERE id = ?2",
                    params![group_id, movie_id],
                )
                .map_err(|e| format!("Failed to tag duplicate movie: {e}"))?;
            }
        }
    }

    Ok(())
}

fn assign_by_title_year(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT LOWER(title) || '|' || COALESCE(CAST(year AS TEXT), ''), GROUP_CONCAT(id)
             FROM movies
             WHERE duplicate_group_id IS NULL
             GROUP BY LOWER(title), year
             HAVING COUNT(*) > 1",
        )
        .map_err(|e| format!("Failed to prepare title/year duplicate query: {e}"))?;

    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| format!("Failed to query title/year duplicates: {e}"))?;

    for row in rows {
        let (key, ids_csv) = row.map_err(|e| format!("Failed to read duplicate row: {e}"))?;
        let group_id = insert_group(conn, &key, "title_year")?;
        for id in ids_csv.split(',') {
            if let Ok(movie_id) = id.trim().parse::<i64>() {
                conn.execute(
                    "UPDATE movies SET duplicate_group_id = ?1 WHERE id = ?2",
                    params![group_id, movie_id],
                )
                .map_err(|e| format!("Failed to tag duplicate movie: {e}"))?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedCopy {
    pub title: String,
    pub path: String,
}

/// Deletes one duplicate movie folder from disk and drops it from the catalog.
/// Refuses the last remaining copy, read-only libraries, and folders that contain another indexed title.
pub fn delete_duplicate_copy(conn: &Connection, movie_id: i64) -> Result<DeletedCopy, String> {
    let (title, relative_path, volume_id, group_id): (String, String, i64, Option<i64>) = conn
        .query_row(
            "SELECT title, relative_path, volume_id, duplicate_group_id FROM movies WHERE id = ?1",
            params![movie_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| format!("Movie {movie_id} not found"))?;

    let group_id = group_id.ok_or("This title is not in a duplicate group")?;
    let copies: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM movies WHERE duplicate_group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count copies: {e}"))?;
    if copies < 2 {
        return Err("Keep at least one copy. This is the last one in the group.".to_string());
    }

    let nested: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM movies
             WHERE volume_id = ?1 AND id != ?2 AND relative_path LIKE ?3 ESCAPE '\\'",
            params![volume_id, movie_id, format!("{}\\%", relative_path.trim_end_matches('\\'))],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check nested titles: {e}"))?;
    if nested > 0 {
        return Err("This folder contains another indexed title. Remove that title first.".to_string());
    }

    let drive_letter: String = conn
        .query_row(
            "SELECT last_drive_letter FROM volumes WHERE id = ?1",
            params![volume_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .map_err(|e| format!("Failed to resolve drive: {e}"))?
        .ok_or("That drive is offline. Connect it before deleting.")?;

    let absolute = format!("{drive_letter}{relative_path}");
    let path = std::path::PathBuf::from(&absolute);
    crate::path_policy::assert_mutation_allowed(&path)?;
    if !path.is_dir() {
        return Err(format!("Movie folder was not found on disk: {absolute}"));
    }

    std::fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete folder: {e}"))?;
    conn.execute("DELETE FROM movies WHERE id = ?1", params![movie_id])
        .map_err(|e| format!("Folder was deleted, but the catalog row could not be removed: {e}"))?;
    rebuild_duplicate_groups(conn)?;

    Ok(DeletedCopy {
        title,
        path: absolute,
    })
}

fn insert_group(conn: &Connection, key: &str, key_type: &str) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO duplicate_groups (key, key_type) VALUES (?1, ?2)",
        params![key, key_type],
    )
    .map_err(|e| format!("Failed to insert duplicate group: {e}"))?;
    Ok(conn.last_insert_rowid())
}
