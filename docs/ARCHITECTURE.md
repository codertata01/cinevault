# CineVault architecture

## Layers

| Layer | Technology | Responsibility |
|-------|------------|----------------|
| UI | React, TypeScript, Vite, Tailwind | Library, duplicates, dialogs |
| IPC | Tauri 2 `invoke` | Thin API to Rust commands |
| Core | Rust | Volumes, scan, SQLite, delete |
| Data | SQLite (`rusqlite`) | Single source of truth in app data dir |

The database is opened in the Rust backend on startup (not `tauri-plugin-sql` in the frontend), so scanning, deletes, and migrations stay in one trust boundary.

## Path model

- **Volume identity:** `serial` + `guid` (never drive letter alone).
- **Stored paths:** `relative_path` on the volume; full path = `{current_letter}{relative_path}` when online.
- **NFO:** TinyMediaManager NFO is canonical metadata.

## Startup sequence

1. Open or create `cinevault.db` under the OS app data directory.
2. Run pending SQL migrations.
3. Register `DbState` for Tauri commands.
4. (Phase 1.3+) Enumerate volumes and detect drive-letter drift before scan.

## UI direction

Dark charcoal, Linear-style productivity UI: library table, filter chips with counts, duplicates comparison, remap and delete confirmations.