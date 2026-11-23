# Implementation plan

| Phase | Task | Status | Commit |
|-------|------|--------|--------|
| 0 | Tauri 2 + React + TS + Vite + Tailwind scaffold | Done | (pending push) |
| 1.1 | SQLite schema + Rust migrations | Done | (pending push) |
| 1.2 | Volume enumeration (Windows) | Done | (pending push) |
| 1.3 | Drive-letter remap prompt | Done | (pending push) |
| 2.1 | Data source management | Done | (pending push) |
| 2.2 | NFO + video scanner | Done (MVP) | (pending push) |
| 2.3 | Offline awareness | Done | (pending push) |
| 3.1 | Duplicate detection | Done | (pending push) |
| 3.2 | Library view (live data) | Done (MVP) | (pending push) |
| 3.3 | Duplicates comparison | Done | (pending push) |
| 4.1 | Safe delete duplicate folder | Done | |
| 5 | Search, export, installer, logging | Partial (search + CSV export; scroll loads pages; filters persist in session) | |
| **UI** | Professional UI — spec locked in [UI_DESIGN_PLAN.md](./UI_DESIGN_PLAN.md) | Spec locked | |
| UI-1 | Shell, Night/Day tokens, buttons, icons, control restyle | Done | |
| UI-2 | Filter chips + library table polish | Done (restyled) | |
| UI-2b | Real poster thumbnails (`poster.jpg`, `convertFileSrc`) | Done | |
| UI-3 | Sources panel + scan progress UI | Done | |
| UI-4 | Duplicates comparison layout (ties 3.3) | Done | |
| UI-5 | Confirm dialogs for folder remove and disk delete | Done | |
| UI-6 | Virtualized table, list, and grid | Done | |

Design system: [`.interface-design/system.md`](../.interface-design/system.md) (interface-design + baseline-ui).

## Task 1.1 notes

- Migration `001_initial.sql`: `volumes`, `data_sources`, `movies`, `duplicate_groups`
- Rust: `src-tauri/src/db/` — open DB, `run_migrations`, unit test
- IPC: `schema_version` for UI health check in status bar