# UI design plan (locked)

Status: **spec locked** — implement from this document. Do not restyle ad hoc.

Grounded in **ui-skills** `ibelick/baseline-ui` and `dammyjay93/interface-design`.  
Token and component measurements: [`.interface-design/system.md`](../.interface-design/system.md).

## Intent

**Who:** Someone at a desk with several movie drives, checking what is indexed and which copy to keep.  
**Task:** Scan folders, browse the catalog, read metadata, spot duplicates.  
**Feel:** Modern private archive. Quiet surfaces, one blue accent, color only for status. Dense enough for hundreds of titles, calm enough to scan for an hour.

Reject: purple gradients, glow, mixed card sizes, every control looking like a text button, header as a toolbar parking lot.

---

## Audit of the current UI

What already works stays. What fights the look gets replaced in **UI-1**.

| Surface | Now | Problem | Locked change |
|---------|-----|---------|----------------|
| Header | Theme, Sync, Add, Scan, Library, Duplicates | Six peers, no focal action | Header holds **title + count + Scan**. Nav moves to the rail. Setup moves next to sources. |
| Theme | Text “Light” / “Dark” | Wrong control for a binary preference | **Icon button** (`Sun` / `Moon`) in the rail footer |
| Library / Duplicates | Segmented control in the header | That pattern is for view modes, not app sections | **Nav items** with icon + label + optional count |
| Kind filters | Pill buttons | Look like tags; active state is a blue border only | **Segmented control** (one selected). Counts stay `tabular-nums` |
| Layout | Text Table / List / Grid | Correct pattern, weak affordance | Same segmented control, **icons only** + `aria-label` |
| Drive | Native `<select>` | Correct control (list grows) | Keep select; style it to 32px height, drive icon in the label |
| Data sources | Always-open card above the table | Steals the catalog | **Disclosure** under the toolbar, collapsed after the first scan |
| Row actions | “Scan” and “Remove” text | Remove should not shout on every row | Scan = small secondary. Remove = **icon button**, danger, confirm |
| Catalog | Table, list, grid, detail panel | Structure is right | Polish badges, sticky header, selected row |
| Grid art | Letter tile | Placeholder | Keep until **UI-2b** posters |
| Status copy | Header string + footer | Easy to miss | **Inline status** under the header action; footer is schema only |
| Color | Zinc + periwinkle `#6e8cff` | Reads generic / slightly purple | **Slate + sky** tokens below |

Keep: `h-dvh`, `cn`, dark/light via `data-theme`, table/list/grid, working filters, drive filter, detail panel, remove-folder confirm (dialog upgrade in UI-5).

---

## Color themes

Two themes only: **Night** (default) and **Day**. Same hue family; only lightness changes. No third accent. No gradients.

| Token | Night | Day | Use |
|-------|-------|-----|-----|
| `--cv-bg` | `#0c0f14` | `#f4f6f8` | App canvas |
| `--cv-surface` | `#141922` | `#ffffff` | Panels, rail |
| `--cv-surface-raised` | `#1c2430` | `#eef2f6` | Hover, selected, segmented thumb |
| `--cv-border` | `#2a3342` | `#d5dde6` | Dividers and control outlines |
| `--cv-foreground` | `#e7edf5` | `#12161c` | Titles and values |
| `--cv-muted` | `#8b97a8` | `#5c6775` | Labels and meta |
| `--cv-accent` | `#3b8cff` | `#1d6fe0` | One primary button and active nav |
| `--cv-accent-muted` | `#2458a3` | `#8eb6ef` | Focus ring, selected chip edge |
| `--cv-success` | `#3cba86` | `#1f8a5b` | Online |
| `--cv-warning` | `#e0a14a` | `#b7791f` | Nested-folder warning, remap |
| `--cv-danger` | `#ef6b6b` | `#d64545` | Offline, remove, failed scan |

Depth: **borders only** (no drop shadows on cards). Dark mode uses a 1px border; light mode may add `shadow-sm` on the detail panel only.

Type: system UI stack already in `index.css` (Segoe UI on Windows). Do not add a display font. Hierarchy from weight and color, not extra sizes. No `tracking-*`.

---

## Shell

```text
┌────┬──────────────────────────────────────────────────────┐
│Film│  Library                          [ Scan library ]  │
│Copy│  1,240 titles · 2 drives online                      │
│    │──────────────────────────────────────────────────────│
│    │  [All|Dup|4K|Off]   Drive ▾          [≡][☰][▦]       │
│    │  ▸ Folders (2)                                           │
│Sun │  ┌────────────────────────────────────────────────┐  │
│    │  │ catalog (table / list / grid)                  │  │
│    │  └────────────────────────────────────────────────┘  │
└────┴──────────────────────────────────────────────────────┘
        detail panel slides in on the right when a title is selected
```

- Rail width **200px**, background `surface`, right border. Items 36px tall, 8px radius, icon 16px + label. Active item: `surface-raised` + accent text. Duplicates shows a count badge.
- Main is the only scroller (`min-h-0 flex-1 overflow-auto`).
- Minimum window **1024×640**. Under 1100px, rail collapses to **56px** icons (`aria-label` on each item). Detail panel becomes a full-height drawer over the catalog.

---

## Control map

Pick the control from the content. Do not use a button style for data, or a nav style for a filter.

| Element | Control | Why |
|---------|---------|-----|
| Library, Duplicates | **Nav item** | Changes the page |
| Table, List, Grid | **Icon segmented control** | One layout of the same list |
| All, Duplicates, 4K, Offline | **Text segmented control** with counts | One filter; short fixed set |
| Drive | **Select** | Options grow with volumes |
| Scan library | **Primary button** + `ScanSearch` | The action on Library |
| Sync drives, Add folder | **Secondary button** | Setup, not the hero |
| Scan one folder | **Small secondary button** | Repeated, still needs a label |
| Remove folder | **Icon button** `FolderMinus`, danger | Destructive; confirm before it runs |
| Theme | **Icon button** | Binary, no label needed on screen |
| Close details | **Icon button** `X` | Dismiss |
| Online / offline / 4K | **Status pill** (not clickable) | It reports state |
| Drive letter, resolution | **Badge** | Short identity, not an action |
| Metadata | **Description list** | Read-only |
| Folders section | **Disclosure** (`details` or button + region) | Secondary to the catalog |
| Remap, delete copy | **Dialog** | A decision that blocks the page |
| Search (phase 5) | **Text input** | Free text, not a chip |

### Button sizes

| Variant | Height | Use |
|---------|--------|-----|
| Primary / secondary | 36px, `px-3`, 14px/500, icon 16px | Header and empty state |
| Small | 28px, `px-2`, 12px/500 | Folder row Scan |
| Icon | 32×32 hit area, 16px glyph, `aria-label` | Theme, remove, close, layout segments |
| Danger | Same sizes, `text-cv-danger`, border `cv-danger` at 40% | Remove, later delete |

Pressed: `active:scale-[0.98]`. Disabled: `opacity-40`. Focus: 2px `accent-muted` ring. No animation beyond that.

---

## Icons (Lucide, one each)

| Meaning | Icon |
|---------|------|
| Library | `Clapperboard` |
| Duplicates | `Copy` |
| Table / list / grid | `Table2` / `List` / `LayoutGrid` |
| Scan library | `ScanSearch` |
| Sync drives | `HardDrive` |
| Add folder | `FolderPlus` |
| Remove folder | `FolderMinus` |
| Theme night / day | `Moon` / `Sun` |
| Close | `X` |
| Online | filled `Circle` in success, 8px |
| Offline | `CircleOff` |
| Drive filter label | `HardDrive` |
| Warning (nested path) | `TriangleAlert` |
| Empty library | `Clapperboard` 28px, muted |

---

## Catalog

**Toolbar row:** kind segmented control on the left, drive select in the middle, layout segmented control on the right. “Showing N titles” sits in the header subtitle, not a second line.

**Table (default):** sticky header. Columns: Title, Year, Resolution, Drive, Status. IMDb stays in the detail panel so the table stays scannable. Selected row uses `surface-raised`.

**List:** one line title, second line `year · drive · resolution`. No poster block until UI-2b.

**Grid:** `minmax(140px, 1fr)`, 2:3 frame. Until posters exist, show title initial on `surface-raised` plus a resolution badge. Do not fake artwork.

**Detail panel (320px):** title, then a definition list — year, resolution, codec, HDR, size, drive, status, IMDb, TMDb, duplicate group, path. Close icon in the header. Empty selection shows no panel (catalog uses the full width).

**Empty catalog:** one sentence and one primary **Add folder** if there are no sources; otherwise **Scan library**.

**Folders disclosure:** summary `Folders · N`. Expanded rows: path (truncate), online pill, Scan, Remove. Warning text for a nested path uses `cv-warning` on that row.

---

## Duplicates view (UI-4, specified now)

Nav item **Duplicates**. No scan button in the header.

- Left: groups as a list (title, year, copy count).
- Right: two (or more) compare cards — path, drive badge, resolution badge, size, online pill.
- Later delete sits on the card that is **not** the keeper, danger button, dialog.

Until UI-4, the current flat list is acceptable but should use the same row and badge styles as Library.

---

## Motion and accessibility

- No entrance animation on the table.
- Scan progress (UI-3) animates **transform** only, under 200ms.
- `prefers-reduced-motion`: no scale on press.
- Icon-only controls have `aria-label`.
- Segmented controls use `aria-pressed`.
- Errors sit next to the control that failed (folder row or dialog), not only in the footer.

---

## Build order (after this spec)

1. **UI-1** — tokens, `Button`, `IconButton`, `SegmentedControl`, rail, header, move actions, disclosure for folders, restyle existing table/list/grid/detail. No new features.
2. **UI-2b** — `poster.jpg` when the drive is online.
3. **UI-3** — scan progress bar and per-source lines.
4. **UI-4 / 3.3** — duplicate comparison.
5. **UI-5 / 4.1** — dialog for remove-from-library now; disk delete later.
6. **UI-6** — virtualize table and list past 500 rows.

UI-2 filter behavior is **done**. UI-1 restyles those controls; it does not rebuild the filter logic.

## Done when UI-1 ships

- Night and Day match the token table.
- One primary button visible on Library.
- Nav, filters, layout, drive, and row actions use the controls in the map.
- Squint test: the catalog is the large region; Scan is the only filled button.
- `npm run build` passes.
