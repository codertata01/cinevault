# Dev and agent testing

## Reference libraries (read-only)

Real movie folders on disk are **reference data only**.

- **Read:** scan NFO/metadata, enumerate for the app.
- **Never:** delete, rename, or modify files inside those roots.
- **Config:** copy `config/reference-libraries.example.json` → `config/reference-libraries.local.json` and set `readOnlyRoots` (gitignored).

Rust enforces this via `path_policy::assert_mutation_allowed` before any future delete/move API (Phase 4).

## Writable test data

If a test needs writes or deletes, **clone a small sample** into the repo:

```powershell
.\scripts\copy-library-sample.ps1 -SourceRoot "E:\Movies" -MaxFolders 3
```

Default destination: `dev/library-copy/` (gitignored). Point CineVault **Add folder** at that copy, not the live library.

## Agent / Cursor token discipline

**Only load folders needed to develop and test** — never whole libraries.

- Do **not** recursively list or paste large trees into chat.
- Use **one sample NFO path** (or two) when debugging parsers.
- Prefer **unit tests** and **`dev/library-copy/`** (3-folder script copy) over live disks.
- Full scans: user runs the app locally; optional `maxNfoFiles` on `scan_data_source` for bounded dev scans (e.g. `10`).