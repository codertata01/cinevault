-- CineVault initial schema (Task 1.1)

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS volumes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    serial TEXT NOT NULL,
    guid TEXT NOT NULL,
    label TEXT,
    filesystem TEXT,
    last_drive_letter TEXT,
    auto_remap INTEGER NOT NULL DEFAULT 0 CHECK (auto_remap IN (0, 1)),
    last_seen TEXT,
    last_remap_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (serial, guid)
);

CREATE INDEX IF NOT EXISTS idx_volumes_serial ON volumes (serial);
CREATE INDEX IF NOT EXISTS idx_volumes_guid ON volumes (guid);
CREATE INDEX IF NOT EXISTS idx_volumes_last_drive_letter ON volumes (last_drive_letter);

CREATE TABLE IF NOT EXISTS data_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    volume_id INTEGER NOT NULL,
    root_path TEXT NOT NULL,
    label TEXT,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (volume_id) REFERENCES volumes (id) ON DELETE CASCADE,
    UNIQUE (volume_id, root_path)
);

CREATE INDEX IF NOT EXISTS idx_data_sources_volume_id ON data_sources (volume_id);

CREATE TABLE IF NOT EXISTS duplicate_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    key_type TEXT NOT NULL CHECK (key_type IN ('imdb', 'tmdb', 'title_year')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (key_type, key)
);

CREATE TABLE IF NOT EXISTS movies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    year INTEGER,
    imdb_id TEXT,
    tmdb_id TEXT,
    relative_path TEXT NOT NULL,
    volume_id INTEGER NOT NULL,
    data_source_id INTEGER,
    resolution_width INTEGER,
    resolution_height INTEGER,
    video_codec TEXT,
    hdr INTEGER NOT NULL DEFAULT 0 CHECK (hdr IN (0, 1)),
    file_size INTEGER,
    nfo_path TEXT,
    watched INTEGER NOT NULL DEFAULT 0 CHECK (watched IN (0, 1)),
    personal_rating INTEGER CHECK (personal_rating IS NULL OR (personal_rating BETWEEN 1 AND 10)),
    notes TEXT,
    duplicate_group_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (volume_id) REFERENCES volumes (id) ON DELETE CASCADE,
    FOREIGN KEY (data_source_id) REFERENCES data_sources (id) ON DELETE SET NULL,
    FOREIGN KEY (duplicate_group_id) REFERENCES duplicate_groups (id) ON DELETE SET NULL,
    UNIQUE (volume_id, relative_path)
);

CREATE INDEX IF NOT EXISTS idx_movies_volume_id ON movies (volume_id);
CREATE INDEX IF NOT EXISTS idx_movies_data_source_id ON movies (data_source_id);
CREATE INDEX IF NOT EXISTS idx_movies_imdb_id ON movies (imdb_id);
CREATE INDEX IF NOT EXISTS idx_movies_tmdb_id ON movies (tmdb_id);
CREATE INDEX IF NOT EXISTS idx_movies_duplicate_group_id ON movies (duplicate_group_id);
CREATE INDEX IF NOT EXISTS idx_movies_year ON movies (year);

INSERT INTO schema_migrations (version) VALUES (1);