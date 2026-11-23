use crate::data_sources;
use crate::scanner::{
    self, DriveLetterCount, MovieListItem, MovieQuery, MovieQueryResult, ScanAllResult, ScanSummary,
};
use crate::state::DbState;
use crate::volumes::{self, VolumeRecord, VolumeSyncResult};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn schema_version(state: State<'_, DbState>) -> Result<i32, String> {
    with_db(&state, |conn| crate::db::current_schema_version(conn))
}

#[tauri::command]
pub fn sync_connected_volumes(state: State<'_, DbState>) -> Result<VolumeSyncResult, String> {
    with_db(&state, volumes::sync_volumes)
}

#[tauri::command]
pub fn list_volumes(state: State<'_, DbState>) -> Result<Vec<VolumeRecord>, String> {
    with_db(&state, volumes::repository::list_volume_records)
}

#[tauri::command]
pub fn apply_volume_remap(
    state: State<'_, DbState>,
    volume_id: i64,
    new_drive_letter: String,
    auto_remap: bool,
) -> Result<VolumeSyncResult, String> {
    with_db(&state, |conn| {
        volumes::repository::apply_remap(&conn, volume_id, &new_drive_letter, auto_remap)?;
        volumes::sync_volumes(conn)
    })
}

#[tauri::command]
pub fn list_data_sources(state: State<'_, DbState>) -> Result<Vec<data_sources::DataSourceRecord>, String> {
    with_db(&state, data_sources::list_data_sources)
}

#[tauri::command]
pub fn add_data_source_from_path(
    state: State<'_, DbState>,
    absolute_path: String,
    label: Option<String>,
) -> Result<data_sources::AddDataSourceResult, String> {
    with_db(&state, |conn| {
        match data_sources::add_data_source_from_absolute_path(
            conn,
            &absolute_path,
            label.as_deref(),
        ) {
            Ok(id) => Ok(id),
            Err(err)
                if err.contains("Sync connected drives first")
                    || err.contains("No known volume") =>
            {
                volumes::sync_volumes(conn)?;
                data_sources::add_data_source_from_absolute_path(
                    conn,
                    &absolute_path,
                    label.as_deref(),
                )
            }
            Err(err) => Err(err),
        }
    })
}

#[tauri::command]
pub fn remove_data_source(
    state: State<'_, DbState>,
    data_source_id: i64,
) -> Result<data_sources::RemoveDataSourceResult, String> {
    with_db(&state, |conn| data_sources::remove_data_source(conn, data_source_id))
}

#[tauri::command]
pub async fn scan_data_source(
    app: AppHandle,
    _state: State<'_, DbState>,
    data_source_id: i64,
    max_nfo_files: Option<u32>,
) -> Result<ScanSummary, String> {
    let app_for_scan = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_scan.state::<DbState>();
        with_db(&state, |conn| {
            scanner::scan_data_source_with_progress(
                conn,
                data_source_id,
                max_nfo_files,
                0,
                1,
                "",
                Some(&|progress| {
                    let _ = app_for_scan.emit("scan-progress", progress);
                }),
            )
        })
    })
    .await
    .map_err(|e| format!("Scan task failed: {e}"))?
}

#[tauri::command]
pub async fn scan_all_data_sources(
    app: AppHandle,
    _state: State<'_, DbState>,
    max_nfo_files: Option<u32>,
) -> Result<ScanAllResult, String> {
    let app_for_scan = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_scan.state::<DbState>();
        with_db(&state, |conn| {
            scanner::scan_all_data_sources(conn, max_nfo_files, Some(&|progress| {
                let _ = app_for_scan.emit("scan-progress", progress);
            }))
        })
    })
    .await
    .map_err(|e| format!("Scan task failed: {e}"))?
}

#[tauri::command]
pub fn delete_duplicate_copy(
    state: State<'_, DbState>,
    movie_id: i64,
) -> Result<crate::duplicates::DeletedCopy, String> {
    with_db(&state, |conn| crate::duplicates::delete_duplicate_copy(conn, movie_id))
}

/// Phase 4 preview: refuses read-only reference library paths.
#[tauri::command]
pub fn validate_delete_target(absolute_folder: String) -> Result<(), String> {
    crate::path_policy::assert_mutation_allowed(std::path::Path::new(&absolute_folder))
}

#[tauri::command]
pub fn list_movies(state: State<'_, DbState>) -> Result<Vec<MovieListItem>, String> {
    with_db(&state, scanner::list_movies)
}

#[tauri::command]
pub fn query_movies(state: State<'_, DbState>, query: MovieQuery) -> Result<MovieQueryResult, String> {
    with_db(&state, |conn| scanner::query_movies(conn, &query))
}

#[tauri::command]
pub fn list_drive_letter_counts(state: State<'_, DbState>) -> Result<Vec<DriveLetterCount>, String> {
    with_db(&state, scanner::list_drive_letter_counts)
}

#[tauri::command]
pub fn export_movies_csv(state: State<'_, DbState>, output_path: String) -> Result<usize, String> {
    with_db(&state, |conn| scanner::export_movies_csv(conn, std::path::Path::new(&output_path)))
}

#[tauri::command]
pub fn library_counts(state: State<'_, DbState>) -> Result<LibraryCounts, String> {
    with_db(&state, |conn| {
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM movies", [], |row| row.get(0))
            .map_err(|e| format!("Count movies failed: {e}"))?;
        let duplicates: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM movies WHERE duplicate_group_id IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Count duplicates failed: {e}"))?;
        let four_k: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM movies m WHERE {}",
                    crate::scanner::resolution_match::SQL_IS_FOUR_K
                ),
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Count 4K failed: {e}"))?;
        let hd_1080: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM movies m WHERE {}",
                    crate::scanner::resolution_match::SQL_IS_1080P
                ),
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Count 1080p failed: {e}"))?;
        let online = crate::volumes::repository::online_volume_ids(conn)?;
        let offline: i64 = if online.is_empty() {
            total
        } else {
            let placeholders = online.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT COUNT(*) FROM movies WHERE volume_id NOT IN ({placeholders})"
            );
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| format!("Count offline failed: {e}"))?;
            stmt.query_row(rusqlite::params_from_iter(online.iter()), |row| row.get(0))
                .map_err(|e| format!("Count offline failed: {e}"))?
        };

        Ok(LibraryCounts {
            all: total,
            duplicates,
            four_k,
            hd_1080,
            offline,
        })
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCounts {
    pub all: i64,
    pub duplicates: i64,
    pub four_k: i64,
    pub hd_1080: i64,
    pub offline: i64,
}

fn with_db<T, F>(state: &State<'_, DbState>, f: F) -> Result<T, String>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, String>,
{
    let conn = state
        .0
        .lock()
        .map_err(|e| format!("Database lock poisoned: {e}"))?;
    f(&conn)
}