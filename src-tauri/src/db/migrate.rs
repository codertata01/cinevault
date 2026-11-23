use rusqlite::{Connection, OptionalExtension};

const MIGRATION_001: &str = include_str!("../../migrations/001_initial.sql");

struct Migration {
    version: i32,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: MIGRATION_001,
}];

pub fn ensure_migrations_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to ensure schema_migrations table: {e}"))
}

pub fn current_schema_version(conn: &Connection) -> Result<i32, String> {
    ensure_migrations_table(conn)?;
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to read schema version: {e}"))
}

fn applied_versions(conn: &Connection) -> Result<std::collections::HashSet<i32>, String> {
    ensure_migrations_table(conn)?;
    let mut stmt = conn
        .prepare("SELECT version FROM schema_migrations")
        .map_err(|e| format!("Failed to prepare version query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, i32>(0))
        .map_err(|e| format!("Failed to query schema versions: {e}"))?;

    let mut versions = std::collections::HashSet::new();
    for row in rows {
        versions.insert(row.map_err(|e| format!("Failed to read migration version: {e}"))?);
    }
    Ok(versions)
}

pub fn run_migrations(conn: &Connection) -> Result<i32, String> {
    ensure_migrations_table(conn)?;
    let applied = applied_versions(conn)?;

    let mut latest = current_schema_version(conn)?;

    for migration in MIGRATIONS {
        if applied.contains(&migration.version) {
            continue;
        }

        let tx = conn
            .unchecked_transaction()
            .map_err(|e| format!("Failed to start migration transaction: {e}"))?;

        tx.execute_batch(migration.sql)
            .map_err(|e| format!("Migration {} failed: {e}", migration.version))?;

        // 001_initial.sql inserts its own version row; later migrations should too or we insert here.
        let recorded: Option<i32> = tx
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                [migration.version],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to verify migration {} record: {e}", migration.version))?;

        if recorded.is_none() {
            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                [migration.version],
            )
            .map_err(|e| {
                format!(
                    "Failed to record migration {} in schema_migrations: {e}",
                    migration.version
                )
            })?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit migration {}: {e}", migration.version))?;

        latest = migration.version;
    }

    Ok(latest)
}