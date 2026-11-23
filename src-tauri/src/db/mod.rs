mod migrate;

use rusqlite::Connection;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

pub use migrate::{current_schema_version, run_migrations};

const DB_FILE_NAME: &str = "cinevault.db";

pub fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {e}"))?;

    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create app data directory {}: {e}", dir.display()))?;

    Ok(dir.join(DB_FILE_NAME))
}

pub fn open_connection(path: &PathBuf) -> Result<Connection, String> {
    Connection::open(path).map_err(|e| format!("Failed to open database {}: {e}", path.display()))
}

pub fn initialize(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    let conn = open_connection(&path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| format!("Failed to enable foreign keys: {e}"))?;
    run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_core_tables() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .expect("foreign keys");
        run_migrations(&conn).expect("migrations");

        let version = current_schema_version(&conn).expect("version");
        assert_eq!(version, 1);

        for table in [
            "volumes",
            "data_sources",
            "movies",
            "duplicate_groups",
            "schema_migrations",
        ] {
            let exists: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("table lookup");
            assert_eq!(exists, 1, "missing table {table}");
        }
    }
}