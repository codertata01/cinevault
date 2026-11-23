# CineVault

Local movie collection manager for multi-drive libraries (NTFS, exFAT, Unraid). NFO-first (TinyMediaManager), volume identity by serial + GUID, duplicate management with safe deletion.

## Stack

- **Desktop:** Tauri 2
- **UI:** React, TypeScript, Vite, Tailwind CSS
- **Backend:** Rust + SQLite (`rusqlite`)

## Develop

```bash
npm install
npm run tauri dev
```

## Database

Migrations run automatically on startup. Schema details: [docs/SCHEMA.md](docs/SCHEMA.md).

## Plan

[docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md)