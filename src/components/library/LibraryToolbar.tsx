import { Download, HardDrive, LayoutGrid, Search, Table2 } from "lucide-react";
import { Button } from "../ui/Button";
import { SegmentedControl } from "../ui/SegmentedControl";
import type { LibraryCounts } from "../../types/api";
import type { LibraryKindFilter, LibraryLayoutMode } from "../../lib/library-filters";

type DriveOption = { letter: string; count: number };

type Props = {
  counts: LibraryCounts;
  kindFilter: LibraryKindFilter;
  driveFilter: string | "all";
  layout: LibraryLayoutMode;
  drives: DriveOption[];
  search: string;
  total: number;
  loading: boolean;
  onSearchChange: (value: string) => void;
  onKindChange: (kind: LibraryKindFilter) => void;
  onDriveChange: (drive: string | "all") => void;
  onLayoutChange: (layout: LibraryLayoutMode) => void;
  onExport: () => void;
};

export function LibraryToolbar({
  counts,
  kindFilter,
  driveFilter,
  layout,
  drives,
  search,
  total,
  loading,
  onSearchChange,
  onKindChange,
  onDriveChange,
  onLayoutChange,
  onExport,
}: Props) {
  return (
    <div className="mb-3 flex flex-col gap-2">
      <div className="flex flex-nowrap items-center gap-2">
        <label className="relative min-w-0 flex-1">
          <Search
            className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-cv-muted"
            aria-hidden
          />
          <input
            type="search"
            value={search}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder="Search title or IMDb…"
            className="h-8 w-full rounded-md border border-cv-border bg-cv-bg py-1 pl-8 pr-3 text-sm text-cv-foreground"
          />
        </label>
        <Button size="sm" className="shrink-0" icon={<Download className="size-3.5" />} onClick={onExport}>
          Export CSV
        </Button>
        <span
          className="min-w-[7.5rem] shrink-0 text-right text-xs tabular-nums text-cv-muted"
          aria-live="polite"
        >
          {loading ? "Loading…" : `${total.toLocaleString()} matches`}
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <SegmentedControl
          label="Filters"
          value={kindFilter}
          onChange={onKindChange}
          items={[
            { id: "all", label: "All", count: counts.all },
            { id: "duplicates", label: "Duplicates", count: counts.duplicates },
            { id: "4k", label: "4K", count: counts.fourK },
            { id: "1080p", label: "1080p", count: counts.hd1080 },
            { id: "offline", label: "Offline", count: counts.offline },
          ]}
        />
        <label className="ml-auto flex items-center gap-2 text-xs text-cv-muted">
          <HardDrive className="size-3.5" aria-hidden />
          <span className="sr-only">Drive</span>
          <select
            aria-label="Drive"
            value={driveFilter}
            onChange={(event) => onDriveChange(event.target.value)}
            className="h-8 rounded-md border border-cv-border bg-cv-bg px-2 text-sm text-cv-foreground"
          >
            <option value="all">All drives</option>
            {drives.map((drive) => (
              <option key={drive.letter} value={drive.letter}>
                {drive.letter} ({drive.count})
              </option>
            ))}
          </select>
        </label>
        <SegmentedControl
          label="Layout"
          iconOnly
          value={layout}
          onChange={onLayoutChange}
          items={[
            { id: "table", label: "Table", icon: <Table2 className="size-3.5" /> },
            { id: "grid", label: "Grid", icon: <LayoutGrid className="size-3.5" /> },
          ]}
        />
      </div>
    </div>
  );
}