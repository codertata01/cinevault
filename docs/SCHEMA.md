# CineVault database schema

SQLite database file: `{app_data_dir}/cinevault.db`

Migrations live in `src-tauri/migrations/` and are applied on app startup via Rust (`rusqlite`).

## schema_migrations

| Column | Type | Notes |
|--------|------|--------|
| version | INTEGER PK | Migration version |
| applied_at | TEXT | UTC timestamp |

## volumes

Persistent identity for physical drives (serial + GUID; drive letter is mutable).

| Column | Type | Notes |
|--------|------|--------|
| id | INTEGER PK | |
| serial | TEXT | Volume serial |
| guid | TEXT | Windows volume GUID |
| label | TEXT | Friendly name |
| filesystem | TEXT | e.g. NTFS, exFAT |
| last_drive_letter | TEXT | Last known letter, e.g. `E:` |
| auto_remap | INTEGER | 0/1 — user opted into automatic remap |
| last_seen | TEXT | Last time volume was online |
| last_remap_at | TEXT | Last letter remap |
| created_at, updated_at | TEXT | |

**Unique:** `(serial, guid)`

## data_sources

Scan roots on a volume (folder added by the user).

| Column | Type | Notes |
|--------|------|--------|
| id | INTEGER PK | |
| volume_id | INTEGER FK → volumes | ON DELETE CASCADE |
| root_path | TEXT | Path relative to volume root |
| label | TEXT | Optional display label |
| enabled | INTEGER | 0/1 |
| created_at, updated_at | TEXT | |

**Unique:** `(volume_id, root_path)`

## duplicate_groups

| Column | Type | Notes |
|--------|------|--------|
| id | INTEGER PK | |
| key | TEXT | IMDb id, TMDB id, or `title\|year` |
| key_type | TEXT | `imdb` \| `tmdb` \| `title_year` |
| created_at | TEXT | |

**Unique:** `(key_type, key)`

## movies

| Column | Type | Notes |
|--------|------|--------|
| id | INTEGER PK | |
| title | TEXT | From NFO |
| year | INTEGER | |
| imdb_id, tmdb_id | TEXT | From NFO uniqueid |
| relative_path | TEXT | Folder path on volume |
| volume_id | INTEGER FK → volumes | |
| data_source_id | INTEGER FK → data_sources | Nullable, SET NULL on delete |
| resolution_width, resolution_height | INTEGER | From streamdetails |
| video_codec | TEXT | |
| hdr | INTEGER | 0/1 |
| file_size | INTEGER | Bytes |
| nfo_path | TEXT | Relative NFO path |
| watched | INTEGER | 0/1 |
| personal_rating | INTEGER | 1–10 or NULL |
| notes | TEXT | |
| duplicate_group_id | INTEGER FK → duplicate_groups | Nullable |
| created_at, updated_at | TEXT | |

**Unique:** `(volume_id, relative_path)`

Offline status is derived at runtime from whether the volume is currently connected (not stored on `movies`).