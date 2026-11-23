mod folder_meta;
mod nfo;
mod query;
pub mod resolution_match;

pub use query::{export_movies_csv, list_drive_letter_counts, query_movies, DriveLetterCount, MovieQuery, MovieQueryResult};

use crate::data_sources::resolve_data_source_root;
use folder_meta::{apply_resolution_from_video_name, parsed_from_folder_name};
use nfo::parse_movie_nfo;
use std::collections::HashSet;
use rusqlite::params;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const VIDEO_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "m2ts", "ts", "mov"];
const MAX_SCAN_ERRORS_REPORTED: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub movies_found: usize,
    pub movies_inserted: usize,
    pub movies_updated: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSourceReport {
    pub data_source_id: i64,
    pub root_path: String,
    pub label: Option<String>,
    pub summary: ScanSummary,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanAllResult {
    pub sources: Vec<ScanSourceReport>,
    pub movies_inserted: usize,
    pub movies_updated: usize,
    pub movies_found: usize,
    pub failed_sources: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub source_index: usize,
    pub source_count: usize,
    pub source_label: String,
    pub current: usize,
    pub total: usize,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieListItem {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub drive_letter: Option<String>,
    pub status: String,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub relative_path: String,
    pub file_size: Option<i64>,
    pub video_codec: Option<String>,
    pub hdr: bool,
    pub duplicate_group_id: Option<i64>,
}

#[allow(dead_code)]
pub fn scan_data_source(
    conn: &Connection,
    data_source_id: i64,
    max_nfo_files: Option<u32>,
) -> Result<ScanSummary, String> {
    scan_data_source_with_progress(conn, data_source_id, max_nfo_files, 0, 1, "", None)
}

pub fn scan_data_source_with_progress(
    conn: &Connection,
    data_source_id: i64,
    max_nfo_files: Option<u32>,
    source_index: usize,
    source_count: usize,
    source_label: &str,
    on_progress: Option<&dyn Fn(&ScanProgress)>,
) -> Result<ScanSummary, String> {
    let source: (i64, String, String, i64) = conn
        .query_row(
            "SELECT ds.id, ds.root_path, COALESCE(ds.label, ''), ds.volume_id
             FROM data_sources ds
             WHERE ds.id = ?1 AND ds.enabled = 1",
            params![data_source_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| format!("Data source {data_source_id} not found or disabled"))?;

    let (drive_letter, volume_id) = resolve_data_source_root(conn, data_source_id)?;

    let absolute_root = format!("{drive_letter}{}", source.1);
    let root = PathBuf::from(&absolute_root);
    if !root.exists() {
        return Err(format!("Data source path does not exist: {absolute_root}"));
    }

    let mut summary = ScanSummary {
        movies_found: 0,
        movies_inserted: 0,
        movies_updated: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    report_progress(
        on_progress,
        source_index,
        source_count,
        source_label,
        0,
        0,
        "Counting folders",
    );
    let movie_folders = collect_movie_folders(&root);
    let total = match max_nfo_files {
        Some(limit) => movie_folders.len().min(limit as usize),
        None => movie_folders.len(),
    };

    for (index, folder) in movie_folders.iter().enumerate() {
        let folder_name = folder
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Movie");
        report_progress(
            on_progress,
            source_index,
            source_count,
            source_label,
            index + 1,
            total.max(1),
            folder_name,
        );
        if let Some(limit) = max_nfo_files {
            if summary.movies_found >= limit as usize {
                break;
            }
        }

        if find_video_file(folder.as_path()).is_none() {
            summary.skipped += 1;
            push_scan_error(
                &mut summary.errors,
                format!("No video file in movie folder {}", folder.display()),
            );
            continue;
        }

        summary.movies_found += 1;

        let nfo_path = find_nfo_file(folder.as_path());
        let video_path = find_video_file(folder.as_path());
        let mut parsed = if let Some(nfo) = nfo_path.as_ref() {
            match parse_movie_nfo(nfo) {
                Ok(mut value) => {
                    let from_folder = parsed_from_folder_name(folder.as_path());
                    value.title.get_or_insert_with(|| {
                        from_folder
                            .title
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string())
                    });
                    if value.year.is_none() {
                        value.year = from_folder.year;
                    }
                    if value.imdb_id.is_none() {
                        value.imdb_id = from_folder.imdb_id;
                    }
                    if value.resolution_height.is_none() {
                        value.resolution_height = from_folder.resolution_height;
                        value.resolution_width = from_folder.resolution_width;
                    }
                    value
                }
                Err(err) => {
                    summary.skipped += 1;
                    push_scan_error(&mut summary.errors, err);
                    continue;
                }
            }
        } else {
            parsed_from_folder_name(folder.as_path())
        };
        if let Some(video) = video_path.as_ref() {
            apply_resolution_from_video_name(&mut parsed, video);
        }

        let relative_movie_path = match relative_from_volume_root(&drive_letter, folder.as_path()) {
            Ok(value) => value,
            Err(err) => {
                summary.skipped += 1;
                push_scan_error(&mut summary.errors, err);
                continue;
            }
        };
        let relative_nfo_path = match nfo_path.as_ref() {
            Some(path) => match relative_from_volume_root(&drive_letter, path) {
                Ok(value) => value,
                Err(err) => {
                    summary.skipped += 1;
                    push_scan_error(&mut summary.errors, err);
                    continue;
                }
            },
            None => String::new(),
        };
        let file_size = video_path
            .as_ref()
            .and_then(|video| std::fs::metadata(video).ok())
            .map(|meta| meta.len() as i64)
            .unwrap_or_default();

        let title = parsed.title.unwrap_or_else(|| "Unknown".to_string());

        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM movies WHERE volume_id = ?1 AND relative_path = ?2",
                params![volume_id, relative_movie_path],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing {
            conn.execute(
                "UPDATE movies SET
                    title = ?1, year = ?2, imdb_id = ?3, tmdb_id = ?4,
                    resolution_width = ?5, resolution_height = ?6,
                    video_codec = ?7, hdr = ?8, file_size = ?9, nfo_path = ?10,
                    data_source_id = ?11, updated_at = datetime('now')
                 WHERE id = ?12",
                params![
                    title,
                    parsed.year,
                    parsed.imdb_id,
                    parsed.tmdb_id,
                    parsed.resolution_width,
                    parsed.resolution_height,
                    parsed.video_codec,
                    if parsed.hdr { 1 } else { 0 },
                    file_size,
                    relative_nfo_path,
                    data_source_id,
                    id
                ],
            )
            .map_err(|e| format!("Failed to update movie: {e}"))?;
            summary.movies_updated += 1;
        } else {
            conn.execute(
                "INSERT INTO movies (
                    title, year, imdb_id, tmdb_id, relative_path, volume_id, data_source_id,
                    resolution_width, resolution_height, video_codec, hdr, file_size, nfo_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    title,
                    parsed.year,
                    parsed.imdb_id,
                    parsed.tmdb_id,
                    relative_movie_path,
                    volume_id,
                    data_source_id,
                    parsed.resolution_width,
                    parsed.resolution_height,
                    parsed.video_codec,
                    if parsed.hdr { 1 } else { 0 },
                    file_size,
                    relative_nfo_path
                ],
            )
            .map_err(|e| format!("Failed to insert movie: {e}"))?;
            summary.movies_inserted += 1;
        }
    }

    let _ = crate::duplicates::rebuild_duplicate_groups(conn);

    Ok(summary)
}

pub fn scan_all_data_sources(
    conn: &Connection,
    max_nfo_files: Option<u32>,
    on_progress: Option<&dyn Fn(&ScanProgress)>,
) -> Result<ScanAllResult, String> {
    let sources: Vec<(i64, String, Option<String>)> = conn
        .prepare(
            "SELECT id, root_path, label FROM data_sources WHERE enabled = 1 ORDER BY id",
        )
        .map_err(|e| format!("Failed to list data sources for scan: {e}"))?
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Failed to query data sources for scan: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read data sources for scan: {e}"))?;

    let mut reports = Vec::with_capacity(sources.len());
    let mut inserted = 0usize;
    let mut updated = 0usize;
    let mut found = 0usize;
    let mut failed = 0usize;

    let source_count = sources.len();
    for (index, (id, root_path, label)) in sources.into_iter().enumerate() {
        let source_label = label.clone().unwrap_or_else(|| root_path.clone());
        match scan_data_source_with_progress(
            conn,
            id,
            max_nfo_files,
            index,
            source_count,
            &source_label,
            on_progress,
        ) {
            Ok(summary) => {
                inserted += summary.movies_inserted;
                updated += summary.movies_updated;
                found += summary.movies_found;
                reports.push(ScanSourceReport {
                    data_source_id: id,
                    root_path,
                    label,
                    summary,
                    error: None,
                });
            }
            Err(err) => {
                failed += 1;
                reports.push(ScanSourceReport {
                    data_source_id: id,
                    root_path,
                    label,
                    summary: ScanSummary {
                        movies_found: 0,
                        movies_inserted: 0,
                        movies_updated: 0,
                        skipped: 0,
                        errors: vec![err.clone()],
                    },
                    error: Some(err),
                });
            }
        }
    }

    Ok(ScanAllResult {
        sources: reports,
        movies_inserted: inserted,
        movies_updated: updated,
        movies_found: found,
        failed_sources: failed,
    })
}

pub fn list_movies(conn: &Connection) -> Result<Vec<MovieListItem>, String> {
  Ok(
        query_movies(
            conn,
            &MovieQuery {
                search: None,
                kind: None,
                drive_letter: None,
                limit: query::MOVIE_PAGE_SIZE,
                offset: 0,
            },
        )?
        .items,
    )
}

fn report_progress(
    on_progress: Option<&dyn Fn(&ScanProgress)>,
    source_index: usize,
    source_count: usize,
    source_label: &str,
    current: usize,
    total: usize,
    title: &str,
) {
    let Some(callback) = on_progress else {
        return;
    };
    let milestone = total == 0 || current <= 1 || current == total || current % 25 == 0;
    if !milestone {
        return;
    }
    callback(&ScanProgress {
        source_index,
        source_count,
        source_label: source_label.to_string(),
        current,
        total,
        title: title.to_string(),
    });
}

fn push_scan_error(errors: &mut Vec<String>, message: String) {
    if errors.len() < MAX_SCAN_ERRORS_REPORTED {
        errors.push(message);
    } else if errors.len() == MAX_SCAN_ERRORS_REPORTED {
        errors.push("Additional scan errors omitted.".to_string());
    }
}

fn collect_movie_folders(root: &Path) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let parent = match path.parent() {
            Some(value) => value,
            None => continue,
        };
        if !parent.starts_with(root) {
            continue;
        }

        let is_nfo = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("nfo"))
            == Some(true);
        let is_video = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| VIDEO_EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)))
            == Some(true);

        if is_nfo || is_video {
            seen.insert(parent.to_path_buf());
        }
    }

    let mut folders: Vec<PathBuf> = seen.into_iter().collect();
    folders.sort_by_cached_key(|path| path.to_string_lossy().to_ascii_lowercase());
    folders
}

fn find_nfo_file(folder: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(folder).ok()?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.eq_ignore_ascii_case("nfo") {
            matches.push(path);
        }
    }
    matches.sort_by_cached_key(|path| path.to_string_lossy().to_ascii_lowercase());
    matches.into_iter().next()
}

fn find_video_file(folder: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(folder).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if VIDEO_EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)) {
            return Some(path);
        }
    }
    None
}

fn drive_root_prefix(drive_letter: &str) -> String {
    let trimmed = drive_letter.trim().trim_end_matches('\\');
    let upper = trimmed.to_ascii_uppercase();
    if upper.ends_with(':') {
        format!("{upper}\\")
    } else if let Some(ch) = upper.chars().next() {
        format!("{ch}:\\")
    } else {
        String::new()
    }
}

fn relative_from_volume_root(drive_letter: &str, path: &Path) -> Result<String, String> {
    let prefix_upper = drive_root_prefix(drive_letter);
    let full = path
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_uppercase();

    let rest = full
        .strip_prefix(&prefix_upper)
        .or_else(|| {
            // Accept `F:` without trailing backslash in the scanned path root.
            full.strip_prefix(prefix_upper.trim_end_matches('\\'))
        })
        .ok_or_else(|| format!("Path {full} is outside volume {prefix_upper}"))?;

    if rest.is_empty() {
        return Ok(String::new());
    }

    Ok(format!("\\{}", rest.trim_start_matches('\\')))
}

#[cfg(test)]
mod path_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn relative_path_under_drive_letter() {
        let path = Path::new(r"F:\XMOVIES\4K\DEADPOOL (2016) {IMDB-TT1431045} [2160P]");
        let rel = relative_from_volume_root("F:", path).unwrap();
        assert_eq!(rel, r"\XMOVIES\4K\DEADPOOL (2016) {IMDB-TT1431045} [2160P]");
    }
}