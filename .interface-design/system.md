# CineVault interface system

Locked with `docs/UI_DESIGN_PLAN.md`. Implement from that spec.

## Direction

Private movie archive on Windows. Modern, quiet, one accent. Night is the default theme; Day is the same palette at higher lightness.

**Signature:** drive-letter badge + resolution badge on every catalog row. Folders read as volumes, not settings.

## Depth

Borders only. Radius: 6px controls, 8px rows and segmented controls, 12px panels. Spacing on a 4px grid. No gradients, no glow, no `tracking-*`.

## Type

14px body. Title 16px/600. Section labels 12px/500 muted (not uppercase tracking). Numbers: `tabular-nums`.

## Themes

See the token table in `docs/UI_DESIGN_PLAN.md` (Night / Day). Semantic colors: accent (actions), success (online), warning (path issues), danger (offline and delete).

## Components

| Pattern | Spec |
|---------|------|
| Button primary | 36px, accent fill, white text, 16px icon, `active:scale-[0.98]` |
| Button secondary | 36px, border, transparent fill |
| Button small | 28px, folder-row Scan |
| Icon button | 32px, `aria-label` |
| Nav item | 36px, icon + label, active = raised surface |
| Segmented control | 32px track, raised thumb for the selected item |
| Select | 32px, for drive only |
| Badge | drive letter and resolution, 12px, mono for the letter |
| Status pill | 8px dot + label, not a button |
| Detail panel | 320px, description list |

## Motion

Press scale only. Respect `prefers-reduced-motion`. Layout shell uses `h-dvh`.
