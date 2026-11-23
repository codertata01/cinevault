import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Clapperboard } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import {
  fetchDriveCounts,
  fetchMoviePage,
  type DriveLetterCount,
} from "../../lib/library-query";
import {
  readStoredFilters,
  readStoredLayout,
  storeFilters,
  storeLayout,
  type LibraryKindFilter,
  type LibraryLayoutMode,
} from "../../lib/library-filters";
import type { DataSourceRecord, LibraryCounts, MovieListItem } from "../../types/api";
import { Button } from "../ui/Button";
import { DataSourcesPanel } from "./DataSourcesPanel";
import { LibraryToolbar } from "./LibraryToolbar";
import { MovieDetailPanel } from "./MovieDetailPanel";
import { MovieGridView } from "./MovieGridView";
import { MovieTableView } from "./MovieTableView";

type Props = {
  counts: LibraryCounts;
  dataSources: DataSourceRecord[];
  busy: boolean;
  refreshToken: number;
  onScan: (sourceId: number) => void;
  onRemove: (source: DataSourceRecord) => void;
  onAdd: () => void;
  onSync: () => void;
  onStatus: (message: string) => void;
};

export function LibraryView({
  counts,
  dataSources,
  busy,
  refreshToken,
  onScan,
  onRemove,
  onAdd,
  onSync,
  onStatus,
}: Props) {
  const storedFilters = readStoredFilters();
  const [kindFilter, setKindFilter] = useState<LibraryKindFilter>(
    storedFilters.kind ?? "all",
  );
  const [driveFilter, setDriveFilter] = useState<string | "all">(
    storedFilters.drive ?? "all",
  );
  const [search, setSearch] = useState(storedFilters.search ?? "");
  const [debouncedSearch, setDebouncedSearch] = useState(storedFilters.search ?? "");
  const [layout, setLayout] = useState<LibraryLayoutMode>(() => readStoredLayout());
  const [selected, setSelected] = useState<MovieListItem | null>(null);
  const [movies, setMovies] = useState<MovieListItem[]>([]);
  const [total, setTotal] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [driveOptions, setDriveOptions] = useState<DriveLetterCount[]>([]);

  useEffect(() => {
    const handle = window.setTimeout(() => setDebouncedSearch(search), 250);
    return () => window.clearTimeout(handle);
  }, [search]);

  useEffect(() => {
    storeFilters({ kind: kindFilter, drive: driveFilter, search });
  }, [kindFilter, driveFilter, search]);

  const loadPage = useCallback(
    async (offset: number, append: boolean) => {
      setLoading(true);
      try {
        const [page, drives] = await Promise.all([
          fetchMoviePage(kindFilter, driveFilter, debouncedSearch, offset),
          fetchDriveCounts(),
        ]);
        setDriveOptions(drives);
        setTotal(page.total);
        setHasMore(page.hasMore);
        setMovies((current) => (append ? [...current, ...page.items] : page.items));
        if (!append) {
          setSelected(null);
        }
      } catch (err) {
        onStatus(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    },
    [kindFilter, driveFilter, debouncedSearch, onStatus],
  );

  useEffect(() => {
    void loadPage(0, false);
  }, [kindFilter, driveFilter, debouncedSearch, refreshToken, loadPage]);

  const loadMore = useCallback(() => {
    if (!hasMore || loading) {
      return;
    }
    void loadPage(movies.length, true);
  }, [hasMore, loading, loadPage, movies.length]);

  const active =
    selected && movies.some((movie) => movie.id === selected.id) ? selected : null;

  async function handleExport() {
    const outputPath = await save({
      defaultPath: "cinevault-library.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!outputPath || Array.isArray(outputPath)) {
      return;
    }
    try {
      const exported = await invoke<number>("export_movies_csv", { outputPath });
      onStatus(`Exported ${exported} movies to ${outputPath}`);
    } catch (err) {
      onStatus(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <LibraryToolbar
        counts={counts}
        kindFilter={kindFilter}
        driveFilter={driveFilter}
        layout={layout}
        drives={driveOptions.map((drive) => ({
          letter: drive.driveLetter,
          count: drive.count,
        }))}
        search={search}
        total={total}
        loading={loading}
        onSearchChange={setSearch}
        onKindChange={setKindFilter}
        onDriveChange={setDriveFilter}
        onLayoutChange={(next) => {
          setLayout(next);
          storeLayout(next);
        }}
        onExport={() => void handleExport()}
      />
      <DataSourcesPanel
        sources={dataSources}
        busy={busy}
        defaultOpen={counts.all === 0}
        onAdd={onAdd}
        onSync={onSync}
        onScan={onScan}
        onRemove={onRemove}
      />
      <div className="flex min-h-0 flex-1 gap-3">
        <section
          aria-label="Movie library"
          className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-xl border border-cv-border bg-cv-surface"
        >
          <div className="min-h-0 flex-1">
            {!loading && movies.length === 0 ? (
              <div className="flex h-full flex-col items-center justify-center gap-3 p-8 text-center">
                <Clapperboard className="size-7 text-cv-muted" aria-hidden />
                <p className="text-balance text-base font-medium">
                  {counts.all === 0 ? "No movies indexed yet" : "No titles match these filters"}
                </p>
                <p className="text-pretty max-w-sm text-sm text-cv-muted">
                  {counts.all === 0
                    ? "Add a folder, then scan the library."
                    : "Try another drive, filter, or search."}
                </p>
                {counts.all === 0 ? (
                  <Button variant="primary" onClick={onAdd} disabled={busy}>
                    Add folder
                  </Button>
                ) : null}
              </div>
            ) : layout === "grid" ? (
              <MovieGridView
                movies={movies}
                selectedId={active?.id ?? null}
                onSelect={setSelected}
                hasMore={hasMore}
                loading={loading}
                onLoadMore={loadMore}
              />
            ) : (
              <MovieTableView
                movies={movies}
                selectedId={active?.id ?? null}
                onSelect={setSelected}
                hasMore={hasMore}
                loading={loading}
                onLoadMore={loadMore}
              />
            )}
          </div>
          {movies.length > 0 ? (
            <div className="flex items-center justify-between gap-2 border-t border-cv-border px-3 py-2 text-xs text-cv-muted">
              <span className="tabular-nums">
                Showing {movies.length.toLocaleString()} of {total.toLocaleString()}
              </span>
              {hasMore ? (
                <Button
                  size="sm"
                  disabled={loading}
                  onClick={() => void loadPage(movies.length, true)}
                >
                  Load more
                </Button>
              ) : null}
            </div>
          ) : null}
        </section>
        {active ? (
          <MovieDetailPanel
            movie={active}
            onClose={() => setSelected(null)}
            onStatus={onStatus}
          />
        ) : null}
      </div>
    </div>
  );
}