use super::resolution_match::{SQL_IS_1080P, SQL_IS_FOUR_K};
use super::MovieListItem;
use rusqlite::{params_from_iter, Connection};

pub const MOVIE_PAGE_SIZE: u32 = 500;
const MAX_LIMIT: u32 = MOVIE_PAGE_SIZE;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieQuery {
    pub search: Option<String>,
    pub kind: Option<String>,
    pub drive_letter: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieQueryResult {
    pub items: Vec<MovieListItem>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveLetterCount {
    pub drive_letter: String,
    pub count: i64,
}

pub fn query_movies(conn: &Connection, query: &MovieQuery) -> Result<MovieQueryResult, String> {
    let limit = query.limit.clamp(1, MAX_LIMIT);
    let offset = query.offset;
    let online = online_volume_ids(conn)?;

    let mut where_parts = Vec::<String>::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(search) = normalize_search(query.search.as_deref()) {
        where_parts.push(
            "(LOWER(m.title) LIKE ?1 OR LOWER(COALESCE(m.imdb_id, '')) LIKE ?1)".to_string(),
        );
        values.push(Box::new(format!("%{search}%")));
    }

    match query.kind.as_deref() {
        Some("duplicates") => where_parts.push("m.duplicate_group_id IS NOT NULL".to_string()),
        Some("4k") => where_parts.push(SQL_IS_FOUR_K.to_string()),
        Some("1080p") => where_parts.push(SQL_IS_1080P.to_string()),
        Some("offline") => {
            if online.is_empty() {
                where_parts.push("1 = 1".to_string());
            } else {
                let placeholders = online.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                where_parts.push(format!("m.volume_id NOT IN ({placeholders})"));
                for id in &online {
                    values.push(Box::new(*id));
                }
            }
        }
        _ => {}
    }

    if let Some(letter) = query
        .drive_letter
        .as_deref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && *value != "all")
    {
        where_parts.push("UPPER(TRIM(v.last_drive_letter)) = UPPER(TRIM(?))".to_string());
        values.push(Box::new(crate::volumes::normalize_drive_letter(letter)));
    }

    let where_sql = if where_parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_parts.join(" AND "))
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM movies m JOIN volumes v ON v.id = m.volume_id {where_sql}"
    );
    let total: i64 = conn
        .query_row(
            &count_sql,
            params_from_iter(values.iter().map(|v| v.as_ref())),
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count movies: {e}"))?;

    let select_sql = format!(
        "SELECT m.id, m.title, m.year, m.resolution_width, m.resolution_height,
                v.last_drive_letter, m.imdb_id, m.tmdb_id, m.relative_path,
                m.file_size, m.video_codec, m.hdr, m.duplicate_group_id, m.volume_id
         FROM movies m
         JOIN volumes v ON v.id = m.volume_id
         {where_sql}
         ORDER BY m.title COLLATE NOCASE
         LIMIT ? OFFSET ?"
    );

    let mut select_values = values;
    select_values.push(Box::new(limit));
    select_values.push(Box::new(offset));

    let mut stmt = conn
        .prepare(&select_sql)
        .map_err(|e| format!("Failed to prepare movie query: {e}"))?;

    let rows = stmt
        .query_map(
            params_from_iter(select_values.iter().map(|v| v.as_ref())),
            |row| map_movie_row(row, &online),
        )
        .map_err(|e| format!("Failed to query movies: {e}"))?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read movies: {e}"))?;

    let fetched = offset as usize + items.len();
    Ok(MovieQueryResult {
        has_more: fetched < total as usize,
        items,
        total: total as usize,
        offset: offset as usize,
        limit: limit as usize,
    })
}

pub fn list_drive_letter_counts(conn: &Connection) -> Result<Vec<DriveLetterCount>, String> {
    use crate::volumes::normalize_drive_letter;

    let mut stmt = conn
        .prepare(
            "SELECT v.last_drive_letter, COALESCE(SUM(src_movies.n), 0)
             FROM data_sources ds
             INNER JOIN volumes v ON v.id = ds.volume_id
             LEFT JOIN (
                 SELECT data_source_id, COUNT(*) AS n
                 FROM movies
                 WHERE data_source_id IS NOT NULL
                 GROUP BY data_source_id
             ) src_movies ON src_movies.data_source_id = ds.id
             WHERE ds.enabled = 1
               AND v.last_drive_letter IS NOT NULL
               AND TRIM(v.last_drive_letter) != ''
             GROUP BY v.last_drive_letter
             ORDER BY v.last_drive_letter",
        )
        .map_err(|e| format!("Failed to prepare drive letter stats: {e}"))?;

    stmt.query_map([], |row| {
        let letter: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        Ok(DriveLetterCount {
            drive_letter: normalize_drive_letter(&letter),
            count,
        })
    })
    .map_err(|e| format!("Failed to query drive letter stats: {e}"))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to read drive letter stats: {e}"))
}

pub fn export_movies_csv(conn: &Connection, path: &std::path::Path) -> Result<usize, String> {
    let online = online_volume_ids(conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT m.title, m.year, m.resolution_width, m.resolution_height,
                    v.last_drive_letter, m.imdb_id, m.relative_path, m.volume_id
             FROM movies m
             JOIN volumes v ON v.id = m.volume_id
             ORDER BY m.title COLLATE NOCASE",
        )
        .map_err(|e| format!("Failed to prepare export: {e}"))?;

    let mut lines = vec![
        "title,year,resolution,drive,status,imdb_id,relative_path".to_string(),
    ];

    let rows = stmt
        .query_map([], |row| {
            let volume_id: i64 = row.get(7)?;
            let width: Option<i32> = row.get(2)?;
            let height: Option<i32> = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i32>>(1)?,
                format_resolution(width, height),
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                if online.contains(&volume_id) {
                    "online"
                } else {
                    "offline"
                },
            ))
        })
        .map_err(|e| format!("Failed to export movies: {e}"))?;

    for row in rows {
        let (title, year, resolution, drive, imdb, rel, status) =
            row.map_err(|e| format!("Failed to read export row: {e}"))?;
        lines.push(format!(
            "{},{},{},{},{},{},{}",
            csv_cell(&title),
            year.map(|y| y.to_string()).unwrap_or_default(),
            csv_cell(&resolution),
            csv_cell(&drive.unwrap_or_default()),
            status,
            csv_cell(&imdb.unwrap_or_default()),
            csv_cell(&rel),
        ));
    }

    std::fs::write(path, lines.join("\n"))
        .map_err(|e| format!("Failed to write CSV {}: {e}", path.display()))?;

    Ok(lines.len().saturating_sub(1))
}

fn normalize_search(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn format_resolution(width: Option<i32>, height: Option<i32>) -> String {
    match (width, height) {
        (Some(w), Some(h)) => format!("{w}x{h}"),
        _ => String::new(),
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn online_volume_ids(conn: &Connection) -> Result<std::collections::HashSet<i64>, String> {
    crate::volumes::repository::online_volume_ids(conn)
        .map(|ids| ids.into_iter().collect())
        .map_err(|e| format!("Failed to resolve online volumes: {e}"))
}

pub fn map_movie_row(
    row: &rusqlite::Row<'_>,
    online: &std::collections::HashSet<i64>,
) -> rusqlite::Result<MovieListItem> {
    let volume_id: i64 = row.get(13)?;
    Ok(MovieListItem {
        id: row.get(0)?,
        title: row.get(1)?,
        year: row.get(2)?,
        resolution_width: row.get(3)?,
        resolution_height: row.get(4)?,
        drive_letter: row.get(5)?,
        imdb_id: row.get(6)?,
        tmdb_id: row.get(7)?,
        relative_path: row.get(8)?,
        file_size: row.get(9)?,
        video_codec: row.get(10)?,
        hdr: row.get::<_, i64>(11)? != 0,
        duplicate_group_id: row.get(12)?,
        status: if online.contains(&volume_id) {
            "online".to_string()
        } else {
            "offline".to_string()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;

    #[test]
    fn paginates_titles() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn.execute(
            "INSERT INTO volumes (serial, guid, label, filesystem, last_drive_letter)
             VALUES ('A', 'G', 'Vol', 'NTFS', 'F:')",
            [],
        )
        .unwrap();
        let volume_id = conn.last_insert_rowid();
        for title in ["Alpha", "Beta", "Gamma"] {
            conn.execute(
                "INSERT INTO movies (title, relative_path, volume_id) VALUES (?1, ?2, ?3)",
                rusqlite::params![title, format!("\\{title}"), volume_id],
            )
            .unwrap();
        }

        let page = query_movies(
            &conn,
            &MovieQuery {
                search: None,
                kind: None,
                drive_letter: None,
                limit: 2,
                offset: 0,
            },
        )
        .unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 2);
        assert!(page.has_more);
    }

    #[test]
    fn four_k_filter_uses_stream_dimensions_not_path() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn.execute(
            "INSERT INTO volumes (serial, guid, label, filesystem, last_drive_letter)
             VALUES ('A', 'G', 'Vol', 'NTFS', 'F:')",
            [],
        )
        .unwrap();
        let volume_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO movies (title, relative_path, volume_id, resolution_height, resolution_width)
             VALUES ('Misfiled path', '\\xMovies\\4K\\Alpha', ?1, NULL, NULL)",
            [volume_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO movies (title, relative_path, volume_id, resolution_height, resolution_width)
             VALUES ('Letterboxed UHD', '\\xMovies\\1080p\\Beta', ?1, 1608, 3840)",
            [volume_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO movies (title, relative_path, volume_id, resolution_height, resolution_width)
             VALUES ('HD only', '\\xMovies\\1080p\\Gamma', ?1, 1080, 1920)",
            [volume_id],
        )
        .unwrap();

        let page = query_movies(
            &conn,
            &MovieQuery {
                search: None,
                kind: Some("4k".to_string()),
                drive_letter: None,
                limit: 50,
                offset: 0,
            },
        )
        .unwrap();
        assert_eq!(page.total, 1);
    }
}